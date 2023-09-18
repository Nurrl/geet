use serde::{Deserialize, Serialize};

/// The configuration for the namespace, currently only some metadata.
#[derive(Debug, Serialize, Deserialize)]
pub struct NamespaceDef {
    name: String,
    description: Option<String>,
}

impl NamespaceDef {
    pub fn new(name: impl Into<String>, description: Option<String>) -> Self {
        Self {
            name: name.into(),
            description: description.map(Into::into),
        }
    }
}
