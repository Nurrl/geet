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
    /// Compute whether the [`Keychain`] contains the provided `key`.
    pub fn contains(&self, key: &PublicKey) -> bool {
        let fingerprint = key.fingerprint(Default::default());

        self.keys
            .iter()
            .any(|k| k.fingerprint(Default::default()) == fingerprint)
    }
}

impl From<&PublicKey> for Keychain {
    fn from(value: &PublicKey) -> Self {
        Self {
            keys: nonempty![value.clone()],
        }
    }
}
