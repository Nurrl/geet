#![feature(result_option_inspect)]

use clap::Parser;
use color_eyre::eyre;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod config;
use config::Cli;

mod repository;
mod server;
mod transport;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Set-up the pretty-printed error handler
    color_eyre::install()?;

    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match Cli::parse() {
        Cli::Server(config) => {
            tracing::info!(
                "Starting up the `geet` daemon in `{}`..",
                config.storage.to_str().unwrap_or("<non-unicode>")
            );

            // Finally configure and start the server
            server::Server::from(config).bind().await
        }
        _ => todo!("The server-side hooks are not implemented yet"),
    }
}
