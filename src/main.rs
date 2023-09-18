use clap::Parser;
use color_eyre::eyre::{self, Context};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use geet::server;

mod config;
use config::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    match Cli::parse() {
        Cli::Server(mut config) => {
            // Set-up the pretty-printed error handler
            color_eyre::install()?;

            // Set-up the log and traces handler
            tracing_subscriber::registry()
                .with(fmt::layer())
                .with(EnvFilter::from_default_env())
                .init();

            config.storage = config
                .storage
                .canonicalize()
                .wrap_err("Error reading the storage directory")?;

            tracing::info!(
                "Starting up the `geet` daemon in `{}`..",
                config.storage.display()
            );

            // Finally configure and start the server
            server::Server::from(config).bind().await
        }
        Cli::Hooks(hook) => hook.run(),
    }
}
