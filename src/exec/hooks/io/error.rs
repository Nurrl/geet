use regex::Regex;
use thiserror::Error;

use furrow::repository::{entries, Id};

use super::Ref;

/// An [`enum@Error`] that can occur while executing a hook.
#[derive(Debug, Error)]
pub enum Error {
    /// When the error is wrapped in a [`Error::Hint`], it will produce
    /// a remote `hint` message and a 0 exit-code.
    #[error(transparent)]
    Hint(#[from] Box<Self>),

    #[error("Unable to process ref update: {0}")]
    RefUpdateParse(parse_display::ParseError),

    #[error("Ref `{0}` may not be deleted.")]
    DeleteRef(Ref),

    #[error("Non fast-forward updates are disabled on `{0}`.")]
    NonFastForward(Ref),

    #[error("The ref name `{0}` does not match {1}.")]
    IllegalRefName(String, Regex),

    #[error("The repository `{0}` is not empty, and thus cannot be removed.")]
    NonEmptyRepository(Id),

    #[error("Unable to parse {0}")]
    EntryParse(#[from] entries::Error),

    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),
}

impl Error {
    /// Acknowledge the error by outputing to `stdout`
    /// and exiting with the correct exit-code.
    pub fn acknowledge(self) -> ! {
        match self {
            Self::Hint(err) => {
                println!("hint: {err}");

                std::process::exit(0);
            }
            _ => {
                println!("error: {self}");

                std::process::exit(1);
            }
        }
    }

    /// Transforms the error into a _hint_,
    /// effectively rendering it non-fatal.
    pub fn into_hint(self) -> Self {
        Self::Hint(self.into())
    }
}
