use std::sync::Arc;

use crate::transport::GitConfig;

use super::{Connection, Server};

/// A factory creating [`Connection`] from the [`Server`] configuration.
#[derive(Debug)]
pub struct Factory {
    server: Arc<Server>,
    gitconfig: Arc<GitConfig>,
}

impl Factory {
    pub fn new(server: Server, gitconfig: GitConfig) -> Self {
        Self {
            server: server.into(),
            gitconfig: gitconfig.into(),
        }
    }
}

impl russh::server::Server for Factory {
    type Handler = Connection;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Connection::new(
            self.server.clone(),
            self.gitconfig.clone(),
            addr.expect("A client connected without an `addr`"),
        )
    }
}
