use thiserror::Error;

#[cfg(doc)]
use super::Source;

/// An [`enum@Error`] that can occur while manipulating an [`Source`].
#[derive(Debug, Error)]

pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error(transparent)]
    ConfigSpanned(format_serde_error::SerdeError),

    #[error(transparent)]
    Config(#[from] serde_yaml::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}
