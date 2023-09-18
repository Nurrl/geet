use std::{collections::HashMap, net::SocketAddr};

use async_trait::async_trait;
use color_eyre::eyre;
use russh::{
    server::{self, Auth, Msg, Response, Session},
    Channel, ChannelId,
};
use russh_keys::key;

use crate::transport::Request;

#[derive(Debug)]
pub struct Connection {
    addr: SocketAddr,
    key: Option<key::PublicKey>,

    requests: HashMap<ChannelId, Request>,
}

impl Connection {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            key: None,
            requests: Default::default(),
        }
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
            proceed_with_methods: None,
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

        self.key = Some(public_key.clone());
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
            self.key
                .as_ref()
                .map(key::PublicKey::fingerprint)
                .unwrap_or_default()
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
            self.key
                .as_ref()
                .map(key::PublicKey::fingerprint)
                .unwrap_or_default()
        );

        if let Some(ref key) = self.key {
            self.requests
                .insert(channel.id(), Request::new(key.clone(), channel));

            Ok((self, true, session))
        } else {
            Ok((self, false, session))
        }
    }

    async fn channel_close(
        mut self,
        channel: ChannelId,
        session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        tracing::info!(
            "Closed channel@{channel} for `{}`",
            self.key
                .as_ref()
                .map(key::PublicKey::fingerprint)
                .unwrap_or_default()
        );

        self.requests.remove(&channel);

        Ok((self, session))
    }

    async fn env_request(
        mut self,
        channel: ChannelId,
        variable_name: &str,
        variable_value: &str,
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        match self.requests.get_mut(&channel) {
            Some(tunnel) => tunnel.push_env(variable_name, variable_value),
            None => {
                session.disconnect(
                    russh::Disconnect::ProtocolError,
                    "Reference to an unknown or closed channel.",
                    "en",
                );
            }
        }

        Ok((self, session))
    }

    async fn exec_request(
        self,
        channel: ChannelId,
        data: &[u8],
        mut session: Session,
    ) -> Result<(Self, Session), Self::Error> {
        match self
            .requests
            .get(&channel)
            .map(|request| request.process(data))
        {
            Some(Ok(_)) => (),
            Some(Err(_)) => {
                session.disconnect(
                    russh::Disconnect::ByApplication,
                    "Unable to process service request.",
                    "en",
                );
            }
            None => {
                session.disconnect(
                    russh::Disconnect::ProtocolError,
                    "Reference to an unknown or closed channel.",
                    "en",
                );
            }
        }

        Ok((self, session))
    }
}
