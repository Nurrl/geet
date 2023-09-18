use std::net::IpAddr;

use russh_keys::PublicKeyBase64;

/// A public-key representation that can come either from disk
/// or from a [`russh_keys::key::PublicKey`].
#[derive(Debug, Clone)]
pub struct Key(openssh_keys::PublicKey);

impl Key {
    pub fn from_russh(
        key: &russh_keys::key::PublicKey,
        user: &str,
        addr: &IpAddr,
    ) -> openssh_keys::errors::Result<Self> {
        Ok(Self(
            format!("{} {} {user}@{addr}", key.name(), key.public_key_base64()).parse()?,
        ))
    }
}

impl std::ops::Deref for Key {
    type Target = openssh_keys::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
