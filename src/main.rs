use clap::Parser;
use color_eyre::eyre::{self, Context};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use geet::{hooks, server};

#[derive(Debug, Parser)]
#[command(multicall = true, rename_all = "kebab-case")]
pub enum Cli {
    #[command(name = env!("CARGO_PKG_NAME"))]
    Server(server::Server),

    #[command(flatten)]
    Hooks(hooks::Hook),
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    match Cli::parse() {
        Cli::Server(mut server) => {
            // Set-up the pretty-printed error handler
            color_eyre::install()?;

            // Set-up the log and traces handler
            tracing_subscriber::registry()
                .with(fmt::layer())
                .with(EnvFilter::from_default_env())
                .init();

            server.storage = server
                .storage
                .canonicalize()
                .wrap_err("Error reading the storage directory")?;

            tracing::info!(
                "Starting up the `geet` daemon in `{}`..",
                server.storage.display()
            );

            // Finally configure and start the server
            server.bind().await
        }
        Cli::Hooks(hook) => hook.run().await,
    }
}
