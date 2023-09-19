use thiserror::Error;

use crate::repository::source;

/// An [`enum@Error`] that can occur while executing a hook.
#[derive(Debug, Error)]
pub enum Error {
    /// When the error is wrapped in a [`Error::Hint`], it will produce
    /// a remote `hint` message and a 0 exit-code.
    #[error(transparent)]
    Hint(#[from] Box<Self>),

    #[error("Unable to process ref update: {0}")]
    RefUpdateParse(parse_display::ParseError),

    #[error("Ref `{0}` may not be deleted")]
    DeleteRef(String),

    #[error("Non fast-forward updates are disabled on `{0}`")]
    NoFastForward(String),

    #[error("Unable to parse source repository: {0}")]
    SourceParse(#[from] source::Error),

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
}
