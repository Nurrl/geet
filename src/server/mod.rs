use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use color_eyre::eyre;
use russh::{MethodSet, SshId};
use russh_keys::key::{KeyPair, SignatureHash};

mod connection;
pub use connection::Connection;

/// A lightweight, self-configured, ssh git remote.
#[derive(Debug, Parser)]
#[command(author, version, about, rename_all = "kebab-case")]
pub struct Server {
    /// The socket addresses to bind, can be supplied multiple times.
    #[arg(short, long, required = true, num_args = 1)]
    pub bind: Vec<SocketAddr>,

    /// The keypairs to use, can be supplied multiple times.
    #[arg(short, long, num_args = 1)]
    pub keypair: Vec<PathBuf>,

    /// Banner text sent to the client on connections.
    #[arg(long)]
    pub banner: Option<String>,

    /// The path of the storage directory.
    pub storage: PathBuf,
}

impl Server {
    pub async fn bind(self) -> eyre::Result<()> {
        let keys = match &self.keypair {
            keypairs if !keypairs.is_empty() => keypairs
                .iter()
                .map(|path| russh_keys::load_secret_key(path, None))
                .collect::<Result<Vec<_>, russh_keys::Error>>()?,
            _ => {
                tracing::warn!("The server has been started without a keypair, random ones will be generated, this is unsafe for production !");

                vec![
                    KeyPair::generate_ed25519().ok_or(eyre::eyre!(
                        "Unable to generate an ed25519 keypair for the server"
                    ))?,
                    KeyPair::generate_rsa(4096, SignatureHash::SHA2_512).ok_or(eyre::eyre!(
                        "Unable to generate a rsa keypair for the server"
                    ))?,
                ]
            }
        };

        let config = russh::server::Config {
            server_id: SshId::Standard(format!(
                "SSH-2.0-{}_{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            )),
            methods: MethodSet::PUBLICKEY,
            auth_banner: self.banner.clone().map(|banner| &*banner.leak()),
            auth_rejection_time: Duration::from_secs(3),
            auth_rejection_time_initial: Some(Duration::ZERO),
            keys,
            inactivity_timeout: Some(Duration::from_secs(3)),
            ..Default::default()
        };

        russh::server::run(config.into(), &*self.bind.clone().leak(), self)
            .await
            .map_err(Into::into)
    }
}

impl russh::server::Server for Server {
    type Handler = connection::Connection;

    fn new_client(&mut self, addr: Option<std::net::SocketAddr>) -> Self::Handler {
        Connection::new(
            self.storage.clone(),
            addr.expect("A client connected without an `addr`"),
        )
    }
}
