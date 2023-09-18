use thiserror::Error;

/// A set of possible errors occuring while manipulation public-keys.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Utf8 error: {0}")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("Parse error: {0}")]
    OpenSsh(#[from] openssh_keys::errors::OpenSSHKeyError),
}
