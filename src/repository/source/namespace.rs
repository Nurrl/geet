use std::{collections::HashMap, ops::Deref};

use nonempty::{nonempty, NonEmpty};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use super::Source;
use crate::{
    repository::{id::Base, Id},
    transport::PubKey,
};

/// An [`Source`] residing in a _non-root_ namespace.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Namespace {
    keys: NonEmpty<PubKey>,

    #[serde(default)]
    repositories: Vec<RepositoryConfig>,
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

    pub fn repository(&self, id: &Id) -> Option<&RepositoryConfig> {
        self.repositories
            .iter()
            .find(|repo| &repo.name == id.repository().deref())
    }
}

impl Source for Namespace {}

/// The configuration for a repository, with some metadata
/// and some technical configuration.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepositoryConfig {
    pub name: Base,
    pub description: Option<String>,
    pub license: Option<String>,

    #[serde(default)]
    pub visibility: Visibility,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub branches: Option<regex::Regex>,
    #[serde_as(as = "Option<serde_with::DisplayFromStr>")]
    pub tags: Option<regex::Regex>,

    #[serde(default)]
    pub branch: HashMap<String, RefConfig>,
}

/// Repository visibility level to a non-owner user.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    /// Only repo owner can clone this repository.
    #[default]
    Private,
    /// Everyone can clone this repository.
    Public,
    /// Everyone can clone this repository, and the repository is read-only.
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]

pub struct RefConfig {
    pub allow_force: bool,
    pub allow_delete: bool,
}

impl RefConfig {
    pub fn protected() -> Self {
        Self {
            allow_force: false,
            allow_delete: false,
        }
    }
}

impl Default for RefConfig {
    fn default() -> Self {
        Self {
            allow_force: true,
            allow_delete: true,
        }
    }
}
