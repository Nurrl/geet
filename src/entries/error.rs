use thiserror::Error;

use super::Entry;

/// An [`struct@Error`] that can occur while manipulating an [`Entry`].
#[derive(Debug, Error)]
#[error("`{path}`: {inner}")]
pub struct Error {
    path: &'static str,
    inner: Kind,
}

impl Error {
    /// Create a new error from it's kind and the `T` type.
    pub fn new<A, T: Entry<A>>(inner: impl Into<Kind>) -> Self {
        Self {
            path: T::PATH,
            inner: inner.into(),
        }
    }

    /// Access the `kind` of this error.
    pub fn kind(&self) -> &Kind {
        &self.inner
    }
}

/// The kind of [`struct@Error`]s that can occur while manipulating an [`Entry`].
#[derive(Debug, Error)]
pub enum Kind {
    /// A _git repository_ error.
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    /// A _config serialization_ error.
    #[error(transparent)]
    ConfigSer(#[from] toml::ser::Error),

    /// A _config deserialization_ error.
    #[error(transparent)]
    ConfigDe(#[from] toml::de::Error),

    /// An _UTF-8_ error.
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}
