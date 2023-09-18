use clap::Parser;
use color_eyre::eyre;

/// The list of available server hooks.
pub const HOOKS: &[&str] = &["pre-receive", "update", "post-receive"];

pub mod params;
use params::Params;

mod post_receive;
mod pre_receive;
mod update;

#[derive(Debug, Parser)]
pub enum Hook {
    /// Execute as a git `pre-receive` hook.
    PreReceive(pre_receive::PreReceive),
    /// Execute as a git `update` hook.
    Update(update::Update),
    /// Execute as a git `post-receive` hook.
    PostReceive(post_receive::PostReceive),
}

impl Hook {
    pub fn run(self) -> eyre::Result<()> {
        match self {
            Hook::PreReceive(hook) => hook.run(),
            Hook::Update(hook) => hook.run(),
            Hook::PostReceive(hook) => hook.run(),
        }
    }
}
