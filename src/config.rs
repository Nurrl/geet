use clap::Parser;

use geet::{hooks, server};

#[derive(Debug, Parser)]
#[command(multicall = true, rename_all = "kebab-case")]
pub enum Cli {
    /// A lightweight, self-configured, ssh git remote.
    #[command(name = env!("CARGO_PKG_NAME"))]
    Server(server::Config),

    #[command(flatten)]
    Hook(hooks::Hook),
}
