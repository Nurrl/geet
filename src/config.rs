use clap::Parser;

use geet::server;

#[derive(Debug, Parser)]
#[command(multicall = true, rename_all = "kebab-case")]
pub enum Cli {
    /// A lightweight, self-configured, ssh git remote.
    #[command(name = env!("CARGO_PKG_NAME"))]
    Server(server::Config),

    /// Execute as a git `pre-receive` hook.
    PreReceive,
    /// Execute as a git `update` hook.
    Update,
    /// Execute as a git `post-receive` hook.
    PostReceive,
}
