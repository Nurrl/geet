use thiserror::Error;

use crate::transport::pkey;

#[cfg(doc)]
use super::Authority;

/// An [`Error`] that can occur while manipulating an [`Authority`].
#[derive(Debug, Error)]

pub enum Error {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Key error: {0}")]
    Key(#[from] pkey::Error),
}
