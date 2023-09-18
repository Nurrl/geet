use std::{sync::Arc, time::Duration};

use russh::{server, MethodSet, SshId};
use russh_keys::key::KeyPair;
use tokio::net::ToSocketAddrs;

use color_eyre::eyre;

mod connection;
pub use connection::Connection;

#[derive(Debug, Clone)]
pub struct Server {
    config: Arc<server::Config>,
}

impl std::default::Default for Server {
    fn default() -> Self {
        let config = server::Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-{}_{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
            methods: MethodSet::PUBLICKEY,
            auth_banner: None,
            auth_rejection_time: Duration::from_secs(3),
            auth_rejection_time_initial: Some(Duration::ZERO),
            keys: vec![KeyPair::generate_ed25519().unwrap()],
            inactivity_timeout: Some(Duration::from_secs(3)),
            ..Default::default()
        };

        Self {
            config: Arc::new(config),
        }
    }
}

impl Server {
    pub async fn bind(self, addrs: impl ToSocketAddrs) -> eyre::Result<()> {
        server::run(self.config.clone(), addrs, self)
            .await
            .map_err(Into::into)
    }
}

impl server::Server for Server {
    type Handler = connection::Connection;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Connection::new(addr.expect("A client connected without an `addr`"))
    }
}
