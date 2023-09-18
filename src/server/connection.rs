use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre;
use russh::{
    server::{self, Auth, Msg, Response, Session},
    Channel, ChannelId, MethodSet,
};
use russh_keys::key;
use tokio::task::JoinHandle;

use super::Server;
use crate::transport::{GitConfig, PubKey, Tunnel};

/// A structure containing connection informations and configuration.
/// implementing [`server::Handler`] to process incoming sessions.
pub struct Connection {
    server: Arc<Server>,
    gitconfig: Arc<GitConfig>,

    addr: SocketAddr,
    key: Option<PubKey>,

    tunnels: HashMap<ChannelId, JoinHandle<()>>,
}

impl Connection {
    pub fn new(server: Arc<Server>, gitconfig: Arc<GitConfig>, addr: SocketAddr) -> Self {
        Self {
            server,
            gitconfig,
            addr,
            key: None,
            tunnels: Default::default(),
        }
    }

    /// Retrieves the client key from the inner connection.
    ///
    /// # Panics
    ///
    /// This will panic if called before the authentication procedure.
    pub fn key(&self) -> &PubKey {
        self.key
            .as_ref()
            .expect("Public key missing from connection context.")
    }
}

#[async_trait]
impl server::Handler for Connection {
    type Error = eyre::Error;

    async fn auth_none(self, user: &str) -> Result<(Self, Auth), Self::Error> {
        tracing::warn!(
            "Unexpected authentication attempt from `{user}@{}` with auth: `none`",
            self.addr
        );

        let auth = Auth::Reject {
            proceed_with_methods: Some(MethodSet::PUBLICKEY),
        };

        Ok((self, auth))
    }

    async fn auth_password(self, user: &str, _password: &str) -> Result<(Self, Auth), Self::Error> {
        tracing::warn!(
            "Unexpected authentication attempt from `{user}@{}` with auth: `password`",
            self.addr
        );

        let auth = Auth::Reject {
            proceed_with_methods: None,
        };

        Ok((self, auth))
    }

    async fn auth_publickey(
        mut self,
        user: &str,
        public_key: &key::PublicKey,
    ) -> Result<(Self, Auth), Self::Error> {
        tracing::info!(
            "Accepting authentication of `{user}@{}` with auth: `public-key` ({})",
            self.addr,
            public_key.fingerprint()
        );

        // Save the client key for further authentication later
        self.key = Some(
            PubKey::from_russh(public_key, user, &self.addr.ip())
                .inspect_err(|err| tracing::error!("Unable to parse client's public-key: {err}"))?,
        );

        Ok((self, Auth::Accept))
    }

    async fn auth_keyboard_interactive(
        self,
        user: &str,
        submethods: &str,
        _response: Option<Response<'async_trait>>,
    ) -> Result<(Self, Auth), Self::Error> {
        tracing::warn!(
            "Unexpected authentication attempt from `{user}@{}` with auth: `keyboard-interactive` ({submethods})",
            self.addr
        );

        let auth = Auth::Reject {
            proceed_with_methods: None,
        };

        Ok((self, auth))
    }

    async fn auth_succeeded(self, session: Session) -> Result<(Self, Session), Self::Error> {
        tracing::info!(
            "Successfully opened SSH session for `{}`",
            self.key().fingerprint()
        );

        Ok((self, session))
    }

    async fn channel_open_session(
        mut self,
        channel: Channel<Msg>,
        session: Session,
    ) -> Result<(Self, bool, Session), Self::Error> {
        tracing::info!(
            "Opening channel #{} for `{}`",
            channel.id(),
            self.key().fingerprint()
        );

        self.tunnels.insert(
            channel.id(),
            Tunnel::new(
                self.server.storage.to_path_buf(),
                self.gitconfig.clone(),
                self.key().clone(),
                channel,
                session.handle(),
            )
            .spawn(),
        );

        Ok((self, true, session))
    }

    async fn channel_close(
        mut self,
        channel: ChannelId,
        session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        tracing::info!(
            "Closed channel #{channel} for `{}`",
            self.key().fingerprint()
        );

        let tunnel = self.tunnels.remove(&channel);
        if let Some(tunnel) = tunnel {
            tunnel.abort();
        }

        Ok((self, session))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        tracing::trace!(
            "Dropping connection for {}, aborting {} tunnels",
            self.addr,
            self.tunnels.len()
        );

        self.tunnels.values().for_each(JoinHandle::abort);
    }
}
