use std::{net::SocketAddr, path::Path};

use assh::side::server::Server;
use assh_auth::handler::publickey::PublicKey;
use assh_connect::Connect;
use color_eyre::eyre;
use futures::TryStreamExt;
use tracing::Instrument;

use super::{
    transport::{GitConfig, Tunnel},
    Socket,
};

/// A structure containing connection informations
/// and configuration to process incoming sessions.
pub struct Connection<'f> {
    session: Connect<Socket, Server>,
    gitconfig: &'f GitConfig,
    storage: &'f Path,

    addr: SocketAddr,
    key: PublicKey,
}

impl<'f> Connection<'f> {
    pub fn new(
        session: Connect<Socket, Server>,
        gitconfig: &'f GitConfig,
        storage: &'f Path,
        addr: SocketAddr,
        key: PublicKey,
    ) -> Self {
        Self {
            session,
            gitconfig,
            storage,
            addr,
            key,
        }
    }

    pub async fn spin(self) -> eyre::Result<()> {
        let Self {
            session,
            gitconfig,
            storage,
            addr,
            key,
        } = self;

        session
            .channel_opens()
            .err_into::<eyre::Error>()
            .try_for_each_concurrent(None, |request| {
                let key = &key;

                async move {
                    tracing::info!(
                        "Opening channel for `{}@{addr}`",
                        key.fingerprint(Default::default())
                    );

                    let channel = request.accept().await?;
                    let tunnel = Tunnel::new(storage, gitconfig, channel, key);

                    tunnel
                        .spin()
                        .instrument(tracing::span!(
                            tracing::Level::INFO,
                            "tunnel",
                            key = %key.fingerprint(Default::default())
                        ))
                        .await?;

                    tracing::info!(
                        "Closing channel for `{}@{addr}`",
                        key.fingerprint(Default::default())
                    );

                    Ok(())
                }
            })
            .await
    }
}

// #[async_trait]
// impl server::Handler for Connection {
//     type Error = eyre::Error;

//     async fn channel_open_session(
//         mut self,
//         channel: Channel<Msg>,
//         session: Session,
//     ) -> Result<(Self, bool, Session), Self::Error> {

//         self.tunnels.insert(
//             channel.id(),
//             Tunnel::new(
//                 self.session.storage.to_path_buf(),
//                 self.gitconfig.clone(),
//                 self.key().clone(),
//                 channel,
//                 session.handle(),
//             )
//             .spawn(),
//         );

//         Ok((self, true, session))
//     }

//     async fn channel_close(
//         mut self,
//         channel: ChannelId,
//         session: Session,
//     ) -> Result<(Self, Session), Self::Error> {

//         let tunnel = self.tunnels.remove(&channel);
//         if let Some(tunnel) = tunnel {
//             tunnel.abort();
//         }

//         Ok((self, session))
//     }
// }
