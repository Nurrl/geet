use serde::{Deserialize, Serialize};

use super::Entry;

impl Entry<()> for Global {
    const PATH: &'static str = "Global.toml";
}

/// An [`Entry`] describing _global_ parameters.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Global {
    /// Server's _self-registration_ policy.
    #[serde(default)]
    pub registration: RegistrationPolicy,
}

impl From<()> for Global {
    fn from(_value: ()) -> Self {
        Self::default()
    }
}

/// Server's _self-registration_ policy.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegistrationPolicy {
    /// Allow users register themselves to the server.
    Allow,

    /// Deny users from registering themselves to the server.
    #[default]
    Deny,
}
