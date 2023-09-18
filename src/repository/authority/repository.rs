use serde::{Deserialize, Serialize};

/// The configuration for a repository, with some metadata
/// and some technical configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryDef {
    name: String,
    description: Option<String>,
    license: Option<String>,
    visibility: Visibility,
    head: Option<String>,
}

/// Repository visibility level to a non-owner user.
#[derive(Debug, Serialize, Deserialize)]
pub enum Visibility {
    /// Everyone can clone this repository.
    Public,
    /// Only repo owner can clone this repository.
    Private,
}
