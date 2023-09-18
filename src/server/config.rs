use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, rename_all = "kebab-case")]
pub struct Config {
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
