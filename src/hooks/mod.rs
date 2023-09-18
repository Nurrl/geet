use std::{io, path::Path};

use clap::Parser;
use color_eyre::eyre;
use strum::{EnumVariantNames, VariantNames};

use crate::repository::Id;

mod error;
pub use error::Error;

pub mod params;
use params::{Params, RefUpdate};

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
    /// Execute the [`Hook`] and exit accordingly.
    pub async fn run(self) -> ! {
        let result = match self {
            Hook::PreReceive(hook) => hook.run().await,
            Hook::Update(hook) => hook.run().await,
            Hook::PostReceive(hook) => hook.run().await,
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

    /// Install server-side hooks for the repository pointed by the [`Id`] in the `storage` path.
    pub fn install(storage: &Path, id: &Id) -> Result<(), eyre::Error> {
        let program = std::env::args().next().expect("The env contains no arg0");
        let hookdir = id.to_path(storage).join("hooks");

        for hook in Self::VARIANTS {
            let link = hookdir.join(hook);

            match std::fs::read_link(&link) {
                Ok(path) if path != Path::new(&program) => {
                    tracing::warn!("Invalidating wrong symlink to `{}`", path.display());

                    std::fs::remove_file(&link)?
                }
                Err(err) if err.kind() == io::ErrorKind::NotFound => (),
                Err(err) => return Err(err.into()),
                _ => continue,
            }

            tracing::trace!("Symlinking `{}` to `{program}`", link.display());

            std::os::unix::fs::symlink(&program, link)?
        }

        Ok(())
    }
}
