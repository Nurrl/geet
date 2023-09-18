use std::net::IpAddr;

use serde_with::{DeserializeFromStr, SerializeDisplay};

mod error;
pub use error::Error;

/// A public-key representation that can come either from disk
/// or from a [`russh_keys::key::PublicKey`].
#[derive(Debug, Clone, PartialEq, SerializeDisplay, DeserializeFromStr)]
pub struct Key(openssh_keys::PublicKey);

impl Key {
    pub fn from_russh(
        key: &russh_keys::key::PublicKey,
        user: &str,
        addr: &IpAddr,
    ) -> openssh_keys::errors::Result<Self> {
        use russh_keys::PublicKeyBase64;

        let name = match key.name() {
            "rsa-sha2-256" | "rsa-sha2-512" => "ssh-rsa",
            name => name,
        };

        Ok(Self(
            format!("{} {} {user}@{addr}", name, key.public_key_base64()).parse()?,
        ))
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.to_key_format())
    }
}

impl std::str::FromStr for Key {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(openssh_keys::PublicKey::parse(s)?))
    }
}

impl std::ops::Deref for Key {
    type Target = openssh_keys::PublicKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
