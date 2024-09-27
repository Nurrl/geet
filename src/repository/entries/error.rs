use thiserror::Error;

#[cfg(doc)]
use super::Entry;

/// An [`enum@Error`] that can occur while manipulating an [`Source`].
#[derive(Debug, Error)]

pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error(transparent)]
    ConfigSer(#[from] toml::ser::Error),

    #[error(transparent)]
    ConfigDe(#[from] toml::de::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}
