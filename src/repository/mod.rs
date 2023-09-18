use std::{io::ErrorKind, path::Path};

use color_eyre::eyre;
use strum::VariantNames;

use crate::hooks;

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
        let repository = git2::Repository::open_bare(id.to_path(storage))?;

        Ok(Self { inner: repository })
    }

    /// Install server-side hooks for the repository pointed by the [`Id`] in the `storage` path.
    pub fn hook(storage: &Path, id: &Id) -> Result<(), eyre::Error> {
        // Ensure the directory exists and is a git repository
        Self::open(storage, id)?;

        let program = std::env::args().next().expect("The env contains no arg0");
        let hookdir = id.to_path(storage).join("hooks");

        for hook in hooks::Hook::VARIANTS {
            let link = hookdir.join(hook);

            match std::fs::read_link(&link) {
                Ok(path) if path != Path::new(&program) => {
                    tracing::warn!("Invalidating wrong symlink to `{}`", path.display());

                    std::fs::remove_file(&link)?
                }
                Err(err) if err.kind() == ErrorKind::NotFound => (),
                Err(err) => return Err(err.into()),
                _ => continue,
            }

            tracing::trace!("Symlinking `{}` to `{program}`", link.display());

            std::os::unix::fs::symlink(&program, link)?
        }

        Ok(())
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
