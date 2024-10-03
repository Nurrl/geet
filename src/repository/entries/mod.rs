//! Configuration of the different functionalities of the server.

use std::path::Path;

use git2::{build::TreeUpdateBuilder, FileMode, Oid};
use serde::{de::DeserializeOwned, Serialize};

use super::Repository;

mod error;
pub use error::Error;

mod global;
pub use global::{Global, RegistrationPolicy};

mod keychain;
pub use keychain::Keychain;

mod repositories;
pub use repositories::{RefConfig, Repositories, Visibility};

/// The trait representing an [`Entry`],
/// which allows R/W operations on a repository storing those kind of informations.
pub trait Entry<Args>: Serialize + DeserializeOwned + From<Args> {
    /// The in-repository path for this [`Entry`].
    const PATH: &'static str;

    /// Load the [`Entry`] from the repository's `HEAD`.
    fn load(repository: &Repository) -> Result<Self, Error> {
        let head = repository.head()?.peel_to_commit()?;

        Self::load_at(repository, head.id())
    }

    /// Load the [`Entry`] from the repository's specified commit [`Oid`].
    fn load_at(repository: &Repository, reference: Oid) -> Result<Self, Error> {
        let commit = repository.find_commit(reference)?;
        let tree = commit.tree()?;
        let blob = tree
            .get_path(Path::new(Self::PATH))?
            .to_object(repository)?
            .peel_to_blob()?;

        let content = std::str::from_utf8(blob.content())?;

        Ok(toml::from_str(content)?)
    }

    fn load_or_init(repository: &Repository, args: Args) -> Result<Self, Error> {
        Self::load(repository).or_else(|err| {
            // Initialize the entry only if it is not found
            match err {
                Error::Git(err)
                    if err.code() == git2::ErrorCode::UnbornBranch
                        || err.code() == git2::ErrorCode::NotFound =>
                {
                    let config = Self::from(args);

                    config
                        .store(
                            repository,
                            &format!("Initialization of the `{}` configuration file", Self::PATH),
                        )
                        .map(|_| config)
                }
                other => Err(other),
            }
        })
    }

    /// Store the [`Entry`] to the repository with a custom commit `message`.
    fn store(&self, repository: &Repository, message: &str) -> Result<(), Error> {
        let blob = repository.blob(toml::to_string_pretty(&self)?.as_bytes())?;
        let signature = git2::Signature::now("furrow", "git@server.commit")?;

        match repository
            .head()
            .ok()
            .map(|reference| reference.peel_to_commit())
            .transpose()?
        {
            Some(parent) => {
                let tree = TreeUpdateBuilder::new()
                    .upsert(Self::PATH, blob, FileMode::Blob)
                    .create_updated(repository, &parent.tree()?)?;
                let tree = repository.find_tree(tree)?;

                repository.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent],
                )?;
            }
            None => {
                let mut treebuilder = repository.treebuilder(None)?;
                treebuilder.insert(Self::PATH, blob, FileMode::Blob.into())?;
                let tree = repository.find_tree(treebuilder.write()?)?;

                repository.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;
            }
        }

        Ok(())
    }
}
