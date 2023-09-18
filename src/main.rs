use clap::Parser;
use color_eyre::eyre::{self, Context};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use geet::{hooks, server};

mod config;
use config::Cli;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let result = match Cli::parse() {
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
                config.storage.to_str().unwrap_or("<non-unicode>")
            );

            // Finally configure and start the server
            Ok(server::Server::from(config).bind().await?)
        }
        Cli::PreReceive => hooks::pre_receive(),
        Cli::Update {
            reference,
            before,
            after,
        } => hooks::update(reference, before, after),
        Cli::PostReceive => hooks::post_receive(),
    };

    // When hooks exit, send back the error to the client
    if let Err(err) = result {
        print!("error: {err}",);
        if let Some(source) = err.source() {
            print!(": {source}");
        }
        println!();

        std::process::exit(1);
    }

    Ok(())
}
