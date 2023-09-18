use serde::{Deserialize, Serialize};

use crate::transport::PubKey;

use super::{namespace::Namespace, Authority};

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
            namespace: Namespace::init(None, key),
            registration: Default::default(),
        }
    }

    pub fn registration(&self) -> bool {
        self.registration
    }
}

impl Authority for Origin {}

impl std::ops::Deref for Origin {
    type Target = Namespace;

    fn deref(&self) -> &Self::Target {
        &self.namespace
    }
}
