use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use color_eyre::eyre;
use russh::{
    server::{self, Auth, Msg, Response, Session},
    Channel, ChannelId, MethodSet,
};
use russh_keys::key;
use tokio::task::JoinHandle;

use crate::{
    config::Config,
    transport::{Key, Request},
};

pub struct Connection {
    config: Arc<Config>,
    addr: SocketAddr,
    key: Option<Key>,

    requests: HashMap<ChannelId, JoinHandle<()>>,
}

impl Connection {
    pub fn new(config: Arc<Config>, addr: SocketAddr) -> Self {
        Self {
            config,
            addr,
            key: None,
            requests: Default::default(),
        }
    }

    /// Retrieves the client key from the inner connection.
    ///
    /// # Panics
    ///
    /// This will panic if called before the authentication procedure.
    pub fn key(&self) -> &Key {
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
            Key::from_russh(public_key, user, &self.addr.ip())
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
            "Opening a `session` channel@{} for `{}`",
            channel.id(),
            self.key().fingerprint()
        );

        self.requests.insert(
            channel.id(),
            Request::new(
                self.key().clone(),
                self.config.storage.clone(),
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
            "Closed channel@{channel} for `{}`",
            self.key().fingerprint()
        );

        let request = self.requests.remove(&channel);
        if let Some(request) = request {
            request.abort();
        }

        Ok((self, session))
    }
}
