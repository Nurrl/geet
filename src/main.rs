#![feature(result_option_inspect)] // see https://github.com/rust-lang/rust/issues/91345.

use color_eyre::eyre::{self, Context};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod config;
mod repository;
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

    let file =
        std::fs::File::open("./geet.yaml").wrap_err("failed to open the configuration file")?;
    let config: config::Config =
        serde_yaml::from_reader(file).wrap_err("failed to parse the configuration file")?;

    tracing::info!("Starting up the `geet` daemon..");

    // Finally configure and start the server
    server::Server::from(config).bind().await
}
