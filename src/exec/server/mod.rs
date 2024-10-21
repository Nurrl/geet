//! Types and structs related to _ssh connection & session handling_.

use std::{net::SocketAddr, path::PathBuf, time::Duration};

use assh::side::server;
use async_compat::{Compat, CompatExt};
use clap::Parser;
use color_eyre::eyre::{self, WrapErr};
use futures::{
    io::{BufReader, BufWriter},
    TryFutureExt,
};
use tokio::net::TcpStream;

mod connection;
use connection::Connection;

mod factory;
use factory::Factory;

mod transport;
use transport::GitConfig;

/// An type alias for the socket used throughout the server implementation.
pub type Socket = BufReader<BufWriter<Compat<TcpStream>>>;

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
    /// Bind and start the server from the configuration.
    pub async fn start(self) -> eyre::Result<()> {
        let storage = self
            .storage
            .canonicalize()
            .wrap_err("Error reading the storage directory")?;

        let keys = match &self.keypair {
            keypairs if !keypairs.is_empty() => keypairs
                .iter()
                .map(|path| server::PrivateKey::read_openssh_file(path).map_err(Into::into))
                .collect::<Result<Vec<_>, assh::Error>>()?,
            _ => {
                let mut rng = rand::thread_rng();

                tracing::warn!("The server has been started without a keypair, random ones will be generated, this is unsafe for production !");

                vec![
                    server::PrivateKey::random(&mut rng, assh::algorithm::Key::Ed25519).map_err(
                        |_| eyre::eyre!("Unable to generate an Ed25519 keypair for the server"),
                    )?,
                    // server::PrivateKey::random(&mut rng, assh::algorithm::Key::Rsa { hash: None })
                    //     .map_err(|_| {
                    //         eyre::eyre!("Unable to generate an RSA keypair for the server")
                    //     })?,
                ]
            }
        };

        tracing::info!(
            "Starting up the `{}` daemon in `{}`..",
            env!("CARGO_PKG_NAME"),
            storage.display()
        );

        let factory = Box::leak(
            Factory::new(
                server::Server {
                    id: server::Id::v2(
                        concat!(env!("CARGO_PKG_NAME"), "_", env!("CARGO_PKG_VERSION")),
                        None::<&str>,
                    ),
                    timeout: Duration::from_secs(3),
                    keys,
                    algorithms: Default::default(),
                },
                {
                    let gitconfig = GitConfig::new(&storage);
                    gitconfig.populate()?;

                    gitconfig
                },
                storage,
            )
            .into(),
        );

        let listener = tokio::net::TcpListener::bind(&*self.bind).await?;
        loop {
            let (stream, addr) = listener.accept().await?;
            let stream = BufReader::new(BufWriter::new(stream.compat()));

            tokio::spawn(
                factory
                    .to_connection(stream, addr)
                    .and_then(Connection::spin)
                    .inspect_err(|err: &eyre::Error| {
                        tracing::error!("Session with client ended up in an error: {err}")
                    }),
            );
        }
    }
}
