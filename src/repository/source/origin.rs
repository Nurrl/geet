use serde::{Deserialize, Serialize};

use super::{namespace::Namespace, Source};
use crate::transport::PubKey;

/// An [`Source`] residing in the _origin_ namespace.
#[derive(Debug, Serialize, Deserialize)]
pub struct Origin {
    #[serde(flatten)]
    namespace: Namespace,

    #[serde(default)]
    registration: bool,
}

impl Origin {
    pub fn init(key: PubKey) -> Self {
        Self {
            namespace: Namespace::init(key),
            registration: Default::default(),
        }
    }

    pub fn registration(&self) -> bool {
        self.registration
    }
}

impl Source for Origin {}

impl std::ops::Deref for Origin {
    type Target = Namespace;

    fn deref(&self) -> &Self::Target {
        &self.namespace
    }
}
