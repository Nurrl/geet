use serde::{Deserialize, Serialize};

use super::Entry;

impl Entry<()> for Global {
    const PATH: &'static str = "Global.toml";
}

/// An [`Entry`] describing _global_ parameters.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Global {
    #[serde(default)]
    pub registration: RegistrationPolicy,
}

impl From<()> for Global {
    fn from(_value: ()) -> Self {
        Self::default()
    }
}

/// The self-registration policy for the server.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistrationPolicy {
    Allow,

    #[default]
    Deny,
}
