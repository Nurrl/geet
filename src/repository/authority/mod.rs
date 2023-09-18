use std::{collections::HashMap, path::Path};

use git2::FileMode;

use super::Repository;
use crate::transport::Key;

mod error;
pub use error::Error;

mod namespace;
pub use namespace::NamespaceDef;

mod repository;
pub use repository::{RepositoryDef, Visibility};

pub const META_CONF_PATH: &str = "meta.yaml";
pub const KEYS_PATH: &str = "keys";
pub const REPOSITORIES_PATH: &str = "repositories";

/// A structure analogue to an _authority_ repository,
/// it's a special kind of repository hosting the servers's
/// configuration and access control.
#[derive(Debug)]
pub struct Authority {
    /// The namespace's meta configuration, `/meta.yaml`.
    meta: NamespaceDef,
    /// The public keys allowed to write to this namespace, `/keys/*.pub`.
    keys: HashMap<Vec<u8>, Key>,
    /// The repositories defined in the namespace, `/repositories/*.yaml`.
    repositories: HashMap<Vec<u8>, RepositoryDef>,
}

impl Authority {
    pub fn init(namespace: Option<impl Into<String>>, key: Key) -> Self {
        Self {
            meta: NamespaceDef::new(
                namespace.map(Into::into).unwrap_or_else(|| "/".into()),
                None,
            ),
            keys: [("default.pub".into(), key)].into_iter().collect(),
            repositories: Default::default(),
        }
    }

    pub fn is_owner(&self, key: &Key) -> bool {
        self.keys.values().any(|k| k == key)
    }

    fn signature() -> Result<git2::Signature<'static>, Error> {
        git2::Signature::now("geet", "git@geet").map_err(Into::into)
    }

    pub fn load(repository: &Repository) -> Result<Self, Error> {
        let head = repository.head()?.peel_to_commit()?;
        let tree = head.tree()?;

        let meta = serde_yaml::from_slice(
            tree.get_path(Path::new(META_CONF_PATH))?
                .to_object(repository)?
                .peel_to_blob()?
                .content(),
        )?;

        let directory = tree
            .get_path(Path::new(KEYS_PATH))?
            .to_object(repository)?
            .peel_to_tree()?;
        let keys = directory
            .into_iter()
            .map(|entry| {
                Ok((
                    entry.name_bytes().to_vec(),
                    Key::from_bytes(entry.to_object(repository)?.peel_to_blob()?.content())?,
                ))
            })
            .collect::<Result<_, Error>>()?;

        let directory = tree
            .get_path(Path::new(REPOSITORIES_PATH))?
            .to_object(repository)?
            .peel_to_tree()?;
        let repositories = directory
            .into_iter()
            .map(|entry| {
                Ok((
                    entry.name_bytes().to_vec(),
                    serde_yaml::from_slice(entry.to_object(repository)?.peel_to_blob()?.content())?,
                ))
            })
            .collect::<Result<_, Error>>()?;

        Ok(Self {
            meta,
            keys,
            repositories,
        })
    }

    pub fn commit(&self, repository: &Repository, message: &str) -> Result<(), Error> {
        let meta = repository.blob(serde_yaml::to_string(&self.meta)?.as_bytes())?;
        let keys = self
            .keys
            .iter()
            .map(|(path, key)| {
                repository
                    .blob(key.to_string().as_bytes())
                    .map(|blob| (path, blob))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;
        let repositories = self
            .repositories
            .iter()
            .map(|(path, repo)| {
                serde_yaml::to_string(repo)
                    .map_err(Error::from)
                    .and_then(|repo| repository.blob(repo.as_bytes()).map_err(Into::into))
                    .map(|repo| (path, repo))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let tree = {
            let mut root = repository.treebuilder(None)?;

            root.insert(META_CONF_PATH, meta, FileMode::Blob.into())?;

            let mut directory = repository.treebuilder(None)?;
            for (path, key) in keys {
                directory.insert(path, key, FileMode::Blob.into())?;
            }
            root.insert(KEYS_PATH, directory.write()?, FileMode::Tree.into())?;

            let mut directory = repository.treebuilder(None)?;
            for (path, repo) in repositories {
                directory.insert(path, repo, FileMode::Blob.into())?;
            }
            root.insert(REPOSITORIES_PATH, directory.write()?, FileMode::Tree.into())?;

            repository.find_tree(root.write()?)?
        };

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
