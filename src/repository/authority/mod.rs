use std::{collections::HashMap, path::PathBuf};

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
#[derive(Debug)]
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
                namespace.map(Into::into).unwrap_or_else(|| "/".into()),
                None,
            ),
            keys: [(DEFAULT_KEY_PATH.into(), key)].into_iter().collect(),
            repositories: Default::default(),
        }
    }

    pub fn load(repository: &Repository) -> Result<Self, git2::Error> {
        let head = repository.head()?.peel_to_commit()?;
        let tree = head.tree()?;

        tracing::error!("{tree:?}");

        unimplemented!()
    }

    fn signature() -> Result<git2::Signature<'static>, git2::Error> {
        git2::Signature::now("geet", "git@geet")
    }

    pub fn commit(&self, repository: &Repository, message: &str) -> eyre::Result<()> {
        let meta = serde_yaml::to_string(&self.meta)?;
        let keys = self
            .keys
            .iter()
            .map(|(path, key)| (path, key.to_key_format()))
            .collect::<HashMap<_, _>>();
        let repositories = self
            .repositories
            .iter()
            .map(|(path, repo)| serde_yaml::to_string(repo).map(|repo| (path, repo)))
            .collect::<Result<HashMap<_, _>, _>>()?;

        let meta = repository.blob(meta.as_bytes())?;
        let keys = keys
            .into_iter()
            .map(|(path, key)| repository.blob(key.as_bytes()).map(|blob| (path, blob)))
            .collect::<Result<HashMap<_, _>, _>>()?;
        let repositories = repositories
            .into_iter()
            .map(|(path, repo)| repository.blob(repo.as_bytes()).map(|blob| (path, blob)))
            .collect::<Result<HashMap<_, _>, _>>()?;

        let mut treebuilder = repository.treebuilder(None)?;

        treebuilder.insert("meta.yaml", meta, 0o100644)?;
        for (path, key) in keys {
            treebuilder.insert(path, key, 0o100644)?;
        }
        for (path, repo) in repositories {
            treebuilder.insert(path, repo, 0o100644)?;
        }

        let tree = repository.find_tree(treebuilder.write()?)?;
        repository.commit(
            Some("HEAD"),
            &Self::signature()?,
            &Self::signature()?,
            message,
            &tree,
            &[],
        )?;

        Ok(())
    }
}
