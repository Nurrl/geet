use nonempty::{nonempty, NonEmpty};
use serde::{Deserialize, Serialize};
use ssh_key::PublicKey;

use super::Entry;

impl Entry<&PublicKey> for Keychain {
    const PATH: &'static str = "Keychain.toml";
}

/// An [`Entry`] describing _keys_ for the namespace.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Keychain {
    keys: NonEmpty<PublicKey>,
}

impl Keychain {
    pub fn contains(&self, key: &PublicKey) -> bool {
        self.keys
            .iter()
            .any(|k| k.fingerprint(Default::default()) == key.fingerprint(Default::default()))
    }
}

impl From<&PublicKey> for Keychain {
    fn from(value: &PublicKey) -> Self {
        Self {
            keys: nonempty![value.clone()],
        }
    }
}
