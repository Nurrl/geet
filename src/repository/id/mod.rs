//! Repository identifier parsing, handling and validation primitives.

use std::{
    path::{self, Path, PathBuf},
    str::FromStr,
};

use super::AUTHORITY_REPOSITORY_NAME;

mod error;
pub use error::Error;

mod name;
pub use name::{Name, REPOSITORY_NAME_EXT};

mod base;
pub use base::Base;

/// The repository type regarding it's [`Id`].
#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    GlobalAuthority,
    LocalAuthority,
    Normal,
}

/// A repository [`Id`] is defined as a path without a leading `/`
/// that does not contain any other component than [`path::Component::Normal`]
/// that are parsed as a [`Base`] and a [`Name`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id {
    namespace: Option<Base>,
    repository: Name,
}

impl Id {
    pub fn new(namespace: Option<Base>, repository: impl Into<Name>) -> Self {
        Self {
            namespace,
            repository: repository.into(),
        }
    }

    /// Get the [`Id`] of the _global authority_ repository.
    pub fn global_authority() -> Self {
        Self {
            namespace: None,
            repository: AUTHORITY_REPOSITORY_NAME,
        }
    }

    pub fn namespace(&self) -> Option<&Base> {
        self.namespace.as_ref()
    }

    pub fn repository(&self) -> &Name {
        &self.repository
    }

    pub fn kind(&self) -> Kind {
        match &self.namespace {
            None if self.is_authority() => Kind::GlobalAuthority,
            Some(_) if self.is_authority() => Kind::LocalAuthority,
            _ => Kind::Normal,
        }
    }

    pub fn is_authority(&self) -> bool {
        self.repository == AUTHORITY_REPOSITORY_NAME
    }

    /// Constructs the config repository [`Id`]
    /// from the current `namespace`:`repository` couple.
    pub fn to_authority(&self) -> Self {
        Self {
            namespace: self.namespace.clone(),
            repository: AUTHORITY_REPOSITORY_NAME,
        }
    }

    /// Converts the current [`Id`] to a [`PathBuf`], in the `storage` path.
    pub fn to_path(&self, storage: &Path) -> PathBuf {
        storage.join(self.to_string())
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Encapsulate in a path and strip any leading `/`
        let path = Path::new(s);
        let path = path.strip_prefix("/").unwrap_or(path);

        let components: Vec<_> = path.components().collect();

        let (namespace, repository) = match components[..] {
            [path::Component::Normal(repository)] => (None, repository.to_str().unwrap().parse()?),
            [path::Component::Normal(namespace), path::Component::Normal(repository)] => (
                Some(namespace.to_str().unwrap().parse()?),
                repository.to_str().unwrap().parse()?,
            ),
            _ => Err(Error::MisformattedPath)?,
        };

        Ok(Self {
            namespace,
            repository,
        })
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(namespace) = &self.namespace {
            f.write_str(namespace)?;
            f.write_str(std::path::MAIN_SEPARATOR_STR)?;
        }
        write!(f, "{}", &self.repository)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("//user/repo.git",
        Id { namespace: Some(Base("user".into())), repository: Name(Base("repo".into())) })]
    #[case("?.git",
            Id { namespace: None, repository: Name(Base("?".into())) })]
    fn it_allows_valid_repositories(#[case] path: &str, #[case] expected: Id) {
        let path = Id::from_str(path).expect(path);

        assert_eq!(path, expected);
    }

    #[rstest]
    #[case("")]
    #[case("/")]
    #[case("..")]
    #[case(".git")]
    #[case("/.git")]
    #[case("~/user/repo.git")]
    #[case("./repo.git")]
    #[case("user/../repo.git")]
    #[case("/user/repo")]
    #[case("/repo")]
    #[case("..git")]
    #[case("toto/..git")]
    #[case(".toto.git")]
    #[case("toto..git")]
    fn it_denies_sketchy_repositories(#[case] path: &str) {
        let _ = Id::from_str(path).unwrap_err();
    }
}
