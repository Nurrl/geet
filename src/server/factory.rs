use std::{net::SocketAddr, path::PathBuf};

use assh_auth::handler;
use color_eyre::eyre;
use tokio::sync::oneshot;

use super::{server::Server, Connection, Socket};
use crate::transport::GitConfig;

/// A factory creating [`Connection`] from the [`Server`] configuration.
#[derive(Debug)]
pub struct Factory {
    config: Server,
    gitconfig: GitConfig,
    storage: PathBuf,
}

impl Factory {
    pub fn new(config: Server, gitconfig: GitConfig, storage: PathBuf) -> Self {
        Self {
            config,
            gitconfig,
            storage,
        }
    }

    pub async fn to_connection(
        &self,
        stream: Socket,
        addr: SocketAddr,
    ) -> eyre::Result<Connection<'_>> {
        let session = assh::Session::new(stream, self.config.clone()).await?;

        let (sender, receiver) = oneshot::channel::<handler::publickey::PublicKey>();
        let sender = &mut Some(sender);
        let session = session
            .handle(
                handler::Auth::new(assh_connect::Service).publickey(|_, key| {
                    sender
                        .take()
                        .expect("Sender has already been consumed at the time")
                        .send(key)
                        .ok();

                    handler::publickey::Response::Accept
                }),
            )
            .await?;
        let key = receiver
            .await
            .expect("Unable to extract the key from the `publickey` authentication");

        Ok(Connection::new(
            session,
            &self.gitconfig,
            &self.storage,
            addr,
            key,
        ))
    }
}
