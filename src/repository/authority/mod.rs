//! Repository, namespace and system-wide config handling.

use std::path::Path;

use git2::FileMode;
use serde::{de::DeserializeOwned, Serialize};

use super::Repository;

mod error;
pub use error::Error;

mod namespace;
pub use namespace::{Namespace, Visibility};

mod origin;
pub use origin::Origin;

fn signature() -> Result<git2::Signature<'static>, Error> {
    git2::Signature::now("geet", "git@server.commit").map_err(Into::into)
}

/// The trait representing an [`Authority`], allows
/// reading and comitting to these special repositories.
pub trait Authority: Serialize + DeserializeOwned {
    /// The in-repository path to the Authority file.
    const PATH: &'static str = "authority.yaml";

    /// Read the [`Authority`] from the `HEAD` of the repository.
    fn read(repository: &Repository) -> Result<Self, Error> {
        let head = repository.head()?.peel_to_commit()?;
        let tree = head.tree()?;

        Ok(serde_yaml::from_slice(
            tree.get_path(Path::new(Self::PATH))?
                .to_object(repository)?
                .peel_to_blob()?
                .content(),
        )?)
    }

    /// Read the [`Authority`] from the provided `commit` in the repository.
    fn read_commit(repository: &Repository, hash: &str) -> Result<Self, Error> {
        let head = repository.find_commit(git2::Oid::from_str(hash)?)?;
        let tree = head.tree()?;

        Ok(serde_yaml::from_slice(
            tree.get_path(Path::new(Self::PATH))?
                .to_object(repository)?
                .peel_to_blob()?
                .content(),
        )?)
    }

    /// Commit the [`Authority`] to the provided repository, with the provided commit `message`.
    fn commit(&self, repository: &Repository, message: &str) -> Result<(), Error> {
        let conf = repository.blob(serde_yaml::to_string(&self)?.as_bytes())?;

        let tree = {
            let mut root = repository.treebuilder(None)?;

            root.insert(Self::PATH, conf, FileMode::Blob.into())?;

            repository.find_tree(root.write()?)?
        };

        let signature = signature()?;
        repository.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?;

        Ok(())
    }
}
