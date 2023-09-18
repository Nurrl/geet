use serde::{Deserialize, Serialize};

use super::Authority;
use crate::{repository::Id, transport::PubKey};

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    name: String,
    keys: Vec<PubKey>,
    repositories: Vec<RepositoryDef>,
}

impl Namespace {
    pub fn init(namespace: Option<String>, key: PubKey) -> Self {
        Self {
            name: namespace.unwrap_or_else(|| ":origin:".into()),
            keys: vec![key],
            repositories: Default::default(),
        }
    }

    pub fn has_key(&self, key: &PubKey) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    pub fn repository(&self, id: &Id) -> Option<&RepositoryDef> {
        self.repositories.iter().find(|repo| {
            repo.name
                .as_str()
                .strip_suffix(".git")
                .unwrap_or(&repo.name)
                == id
                    .repository()
                    .strip_suffix(".git")
                    .unwrap_or(id.repository())
        })
    }
}

impl Authority for Namespace {}

/// The configuration for a repository, with some metadata
/// and some technical configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDef {
    name: String,
    description: Option<String>,
    license: Option<String>,
    visibility: Visibility,
}

impl RepositoryDef {
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// Repository visibility level to a non-owner user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    /// Only repo owner can clone this repository.
    Private,
    /// Everyone can clone this repository.
    Public,
    /// Everyone can clone this repository, and the repository is read-only.
    Archive,
}
