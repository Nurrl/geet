use std::path::Path;

use git2::RepositoryOpenFlags;

/// Defines the default git `HEAD` ref when creating a new repository
pub const DEFAULT_HEAD_REF: &str = "refs/heads/main";

/// The name of the authority repository in the repository root
/// and in repository namespaces.
pub const AUTHORITY_REPOSITORY_NAME: &str = "?.git";

pub mod id;
pub use id::Id;

pub mod authority;

/// A handle to a bare repository.
pub struct Repository {
    inner: git2::Repository,
}

impl Repository {
    /// Initialize the repository pointed by the [`Id`] in the `storage` path.
    pub fn init(storage: &Path, id: &Id) -> Result<Self, git2::Error> {
        let repository = git2::Repository::init_bare(id.to_path(storage))?;
        repository.set_head(DEFAULT_HEAD_REF)?;

        Ok(Self { inner: repository })
    }

    /// Open the repository pointed by the [`Id`] in the `storage` path.
    pub fn open(storage: &Path, id: &Id) -> Result<Self, git2::Error> {
        let repository = git2::Repository::open_ext(
            id.to_path(storage),
            RepositoryOpenFlags::NO_SEARCH
                | RepositoryOpenFlags::BARE
                | RepositoryOpenFlags::NO_DOTGIT,
            &[] as &[&std::ffi::OsStr],
        )?;

        Ok(Self { inner: repository })
    }

    /// Open the repository pointed by the envs, used in hooks.
    pub fn open_from_hook(storage: &Path, id: &Id) -> Result<Self, git2::Error> {
        let repository = git2::Repository::open_ext(
            id.to_path(storage),
            RepositoryOpenFlags::NO_SEARCH
                | RepositoryOpenFlags::BARE
                | RepositoryOpenFlags::NO_DOTGIT
                | RepositoryOpenFlags::FROM_ENV,
            &[] as &[&std::ffi::OsStr],
        )?;

        Ok(Self { inner: repository })
    }
}

impl std::ops::Deref for Repository {
    type Target = git2::Repository;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl From<git2::Repository> for Repository {
    fn from(value: git2::Repository) -> Self {
        Self { inner: value }
    }
}
