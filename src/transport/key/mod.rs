use std::net::IpAddr;

mod error;
pub use error::Error;

/// A public-key representation that can come either from disk
/// or from a [`russh_keys::key::PublicKey`].
#[derive(Debug, Clone, PartialEq)]
pub struct Key(openssh_keys::PublicKey);

impl Key {
    pub fn from_russh(
        key: &russh_keys::key::PublicKey,
        user: &str,
        addr: &IpAddr,
    ) -> openssh_keys::errors::Result<Self> {
        use russh_keys::PublicKeyBase64;

        Ok(Self(
            format!("{} {} {user}@{addr}", key.name(), key.public_key_base64()).parse()?,
        ))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        Ok(Self(openssh_keys::PublicKey::parse(std::str::from_utf8(
            bytes,
        )?)?))
    }
}

impl std::ops::Deref for Key {
    type Target = openssh_keys::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_key_format())
    }
}
