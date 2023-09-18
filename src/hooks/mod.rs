use clap::Parser;
use strum::EnumVariantNames;

mod error;
pub use error::Error;

pub mod params;
use params::Params;

mod post_receive;
mod pre_receive;
mod update;

/// The collection of git hooks defined for this remote.
#[derive(Debug, Parser, EnumVariantNames)]
#[command(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Hook {
    /// Execute as a git `pre-receive` hook.
    PreReceive(pre_receive::PreReceive),
    /// Execute as a git `update` hook.
    Update(update::Update),
    /// Execute as a git `post-receive` hook.
    PostReceive(post_receive::PostReceive),
}

impl Hook {
    pub fn run(self) -> ! {
        let result = match self {
            Hook::PreReceive(hook) => hook.run(),
            Hook::Update(hook) => hook.run(),
            Hook::PostReceive(hook) => hook.run(),
        };

        match result {
            Err(Error::Err(err)) => {
                print!("error: {err}");
                if let Some(source) = err.source() {
                    print!(": {source}");
                }
                println!();

                std::process::exit(1);
            }
            Err(Error::Warn(err)) => {
                print!("warning: {err}");
                if let Some(source) = err.source() {
                    print!(": {source}");
                }
                println!();
            }
            Ok(_) => (),
        }

        std::process::exit(0);
    }
}
