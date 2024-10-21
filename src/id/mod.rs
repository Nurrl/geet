//! Repository _identifier_ parsing, handling and validation primitives.

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
pub use base::{Base, AUTHORIZED_BASENAMES};

/// The repository type regarding it's [`Id`].
#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    /// The [`Id`] points to a _global authority_.
    GlobalAuthority,

    /// The [`Id`] points to a _local authority_.
    LocalAuthority,

    /// The [`Id`] points to a _normal repository_.
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
    /// Create an [`Id`] from a `namespace` and a `repository`.
    pub fn new(namespace: Option<Base>, repository: impl Into<Name>) -> Self {
        Self {
            namespace,
            repository: repository.into(),
        }
    }

    /// Create an [`Id`] pointing to the _global authority_ repository.
    pub fn global_authority() -> Self {
        Self {
            namespace: None,
            repository: AUTHORITY_REPOSITORY_NAME,
        }
    }

    /// Access _namespace_ pointed by this [`Id`].
    pub fn namespace(&self) -> Option<&Base> {
        self.namespace.as_ref()
    }

    /// Access _repository_ pointed by this [`Id`].
    pub fn repository(&self) -> &Name {
        &self.repository
    }

    /// Compute _kind_ of repository pointed by this [`Id`].
    pub fn kind(&self) -> Kind {
        match &self.namespace {
            None if self.is_authority() => Kind::GlobalAuthority,
            Some(_) if self.is_authority() => Kind::LocalAuthority,
            _ => Kind::Normal,
        }
    }

    /// Whether this [`Id`] points to an _authority_.
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
            [path::Component::Normal(repository)] => (
                None,
                repository
                    .to_str()
                    .expect("Path component is not UTF-8")
                    .parse()?,
            ),
            [path::Component::Normal(namespace), path::Component::Normal(repository)] => (
                Some(
                    namespace
                        .to_str()
                        .expect("Path component is not UTF-8")
                        .parse()?,
                ),
                repository
                    .to_str()
                    .expect("Path component is not UTF-8")
                    .parse()?,
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
    #[case("_.git",
            Id { namespace: None, repository: Name(Base("_".into())) })]
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
    #[case("/user.git/repo.git")]
    #[case("/repo")]
    #[case("..git")]
    #[case("toto/..git")]
    #[case(".toto.git")]
    #[case("toto..git")]
    fn it_denies_sketchy_repositories(#[case] path: &str) {
        let _ = Id::from_str(path).expect_err("The `id` was malformed, but didn't error");
    }
}
