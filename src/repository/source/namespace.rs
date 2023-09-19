use std::ops::Deref;

use nonempty::{nonempty, NonEmpty};
use serde::{Deserialize, Serialize};

use super::Source;
use crate::{
    repository::{id::Base, Id},
    transport::PubKey,
};

/// An [`Source`] residing in a _non-root_ namespace.
#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    keys: NonEmpty<PubKey>,
    repositories: Vec<RepositoryDef>,
}

impl Namespace {
    pub fn init(key: PubKey) -> Self {
        Self {
            keys: nonempty![key],
            repositories: Default::default(),
        }
    }

    pub fn has_key(&self, key: &PubKey) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    pub fn repository(&self, id: &Id) -> Option<&RepositoryDef> {
        self.repositories
            .iter()
            .find(|repo| &repo.name == id.repository().deref())
    }
}

impl Source for Namespace {}

/// The configuration for a repository, with some metadata
/// and some technical configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryDef {
    name: Base,
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
