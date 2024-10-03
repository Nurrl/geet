use thiserror::Error;

use super::Entry;

/// An [`struct@Error`] that can occur while manipulating an [`Entry`].
#[derive(Debug, Error)]
#[error("`{path}`: {inner}")]
pub struct Error {
    path: &'static str,
    inner: ErrorKind,
}

impl Error {
    pub fn new<A, T: Entry<A>>(inner: impl Into<ErrorKind>) -> Self {
        Self {
            path: T::PATH,
            inner: inner.into(),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.inner
    }
}

/// The kind of [`struct@Error`]s that can occur while manipulating an [`Entry`].
#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error(transparent)]
    ConfigSer(#[from] toml::ser::Error),

    #[error(transparent)]
    ConfigDe(#[from] toml::de::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}
