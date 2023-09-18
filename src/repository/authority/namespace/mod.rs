use serde::{Deserialize, Serialize};

use super::Authority;
use crate::transport::Key;

#[derive(Debug, Serialize, Deserialize)]
pub struct Namespace {
    name: String,
    keys: Vec<Key>,
    repositories: Vec<Repository>,
}

/// The configuration for a repository, with some metadata
/// and some technical configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
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

impl Namespace {
    pub fn init(namespace: Option<String>, key: Key) -> Self {
        Self {
            name: namespace.unwrap_or_else(|| ":origin:".into()),
            keys: vec![key],
            repositories: Default::default(),
        }
    }

    pub fn has_key(&self, key: &Key) -> bool {
        self.keys.iter().any(|k| k == key)
    }
}

impl Authority for Namespace {}
