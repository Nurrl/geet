use serde::{Deserialize, Serialize};
use ssh_key::PublicKey;

use super::{namespace::Namespace, Source};

/// An [`Source`] residing in the _origin_ namespace.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Origin {
    #[serde(flatten)]
    source: Namespace,

    #[serde(default)]
    allow_registration: bool,
}

impl Origin {
    pub fn init(key: PublicKey) -> Self {
        Self {
            allow_registration: Default::default(),
            source: Namespace::init(key),
        }
    }

    pub fn allow_registration(&self) -> bool {
        self.allow_registration
    }
}

impl Source for Origin {}

impl std::ops::Deref for Origin {
    type Target = Namespace;

    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
