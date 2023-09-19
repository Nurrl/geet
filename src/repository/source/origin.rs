use serde::{Deserialize, Serialize};

use super::{namespace::Namespace, Source};
use crate::transport::PubKey;

/// An [`Source`] residing in the _origin_ namespace.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Origin {
    #[serde(default)]
    allow_registration: bool,

    #[serde(flatten)]
    namespace: Namespace,
}

impl Origin {
    pub fn init(key: PubKey) -> Self {
        Self {
            allow_registration: Default::default(),
            namespace: Namespace::init(key),
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
        &self.namespace
    }
}
