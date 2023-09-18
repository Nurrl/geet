#![feature(result_option_inspect)] // see https://github.com/rust-lang/rust/issues/91345.

use color_eyre::eyre;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod server;
mod transport;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> eyre::Result<()> {
    // Set-up the pretty-printed error handler
    color_eyre::install()?;

    // Set-up the log and traces handler
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting up the `geet` daemon..");

    // Finally configure and start the server
    server::Server::default().bind(("0.0.0.0", 2222)).await
}
