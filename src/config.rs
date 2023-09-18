use clap::Parser;

use geet::{hooks, server};

#[derive(Debug, Parser)]
#[command(multicall = true, rename_all = "kebab-case")]
pub enum Cli {
    #[command(name = env!("CARGO_PKG_NAME"))]
    Server(server::Config),

    #[command(flatten)]
    Hooks(hooks::Hook),
}
