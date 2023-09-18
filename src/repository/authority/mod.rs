use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use color_eyre::eyre;

use crate::transport::Key;

use super::Repository;

mod namespace;
pub use namespace::NamespaceDef;

mod repository;
pub use repository::{RepositoryDef, Visibility};

pub const DEFAULT_KEY_PATH: &str = "default.pub";

/// A structure analogue to an _authority_ repository,
/// it's a special kind of repository hosting the servers's
/// configuration and access control.
pub struct Authority {
    /// The namespace's meta configuration, `/meta.yaml`.
    meta: NamespaceDef,
    /// The public keys allowed to write to this namespace, `/keys/*.pub`.
    keys: HashMap<PathBuf, Key>,
    /// The repositories defined in the namespace, `/repositories/*.yaml`.
    repositories: HashMap<PathBuf, RepositoryDef>,
}

impl Authority {
    pub fn init(namespace: Option<impl Into<String>>, key: Key) -> Self {
        Self {
            meta: NamespaceDef::new(
                namespace
                    .map(Into::into)
                    .unwrap_or_else(|| "initial".into()),
                None,
            ),
            keys: [(DEFAULT_KEY_PATH.into(), key)].into_iter().collect(),
            repositories: Default::default(),
        }
    }

    pub fn load(repository: &Repository) -> eyre::Result<Self> {
        let head = repository.head()?.peel_to_commit()?;

        tracing::error!("{head:?}");

        unimplemented!()
    }

    pub fn store(&self, repository: &Repository) -> eyre::Result<()> {
        let meta = serde_yaml::to_string(&self.meta)?;
        let keys = ();

        Ok(())
    }
}
