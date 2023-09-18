use std::{sync::Arc, time::Duration};

use color_eyre::eyre;
use russh::{MethodSet, SshId};
use russh_keys::key::KeyPair;

use crate::config;

mod connection;
pub use connection::Connection;

#[derive(Debug)]
pub struct Server {
    config: Arc<config::Config>,
}

impl From<config::Config> for Server {
    fn from(value: config::Config) -> Self {
        Self {
            config: value.into(),
        }
    }
}

impl Server {
    pub async fn bind(self) -> eyre::Result<()> {
        let keys = match self.config.keys {
            Some(ref keys) => keys
                .iter()
                .map(|path| russh_keys::load_secret_key(path, None))
                .collect::<Result<Vec<_>, russh_keys::Error>>()?,
            None => {
                tracing::warn!("The server has been started without a keypair, a random one will be generated, this is unsafe for production !");

                vec![KeyPair::generate_ed25519().ok_or(eyre::eyre!(
                    "Unable to generate a random keypair for the server"
                ))?]
            }
        };

        let config = russh::server::Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-{}_{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
            methods: MethodSet::PUBLICKEY,
            auth_banner: self
                .config
                .banner
                .clone()
                .map(|banner| banner.leak() as &'static str),
            auth_rejection_time: Duration::from_secs(3),
            auth_rejection_time_initial: Some(Duration::ZERO),
            keys,
            inactivity_timeout: Some(Duration::from_secs(3)),
            ..Default::default()
        };

        russh::server::run(config.into(), self.config.address, self)
            .await
            .map_err(Into::into)
    }
}

impl russh::server::Server for Server {
    type Handler = connection::Connection;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Connection::new(
            self.config.clone(),
            addr.expect("A client connected without an `addr`"),
        )
    }
}
