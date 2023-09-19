use thiserror::Error;

/// A set of possible errors occuring while manipulation public-keys.
#[derive(Debug, Error)]
#[error("Cannot parse key, {0}")]
pub enum Error {
    Utf8(#[from] std::str::Utf8Error),
    OpenSsh(#[from] openssh_keys::errors::OpenSSHKeyError),
}
