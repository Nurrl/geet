use std::path::Path;

use color_eyre::eyre;

/// Defines the default git `HEAD` ref when creating a new repository
pub const DEFAULT_HEAD_REF: &str = "refs/heads/main";

/// The name of the authority repository in the repository root
/// and in repository namespaces.
pub const AUTHORITY_REPOSITORY_NAME: &str = "~.git";

mod id;
pub use id::Id;

pub mod authority;
pub use authority::Authority;

/// A handle to a bare repository.
pub struct Repository {
    repository: git2::Repository,
}

impl Repository {
    pub fn open(storage: &Path, id: &Id) -> eyre::Result<Self> {
        let repository = git2::Repository::open_bare(id.to_path(storage))?;

        Ok(Self { repository })
    }
}

impl std::ops::Deref for Repository {
    type Target = git2::Repository;

    fn deref(&self) -> &Self::Target {
        &self.repository
    }
}
