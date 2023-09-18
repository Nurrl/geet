use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{self, Path, PathBuf},
    str::FromStr,
};

use color_eyre::eyre;

use super::AUTHORITY_REPOSITORY_NAME;

/// The standard extension for git repositories.
const REPOSITORY_NAME_EXT: &str = ".git";

/// A repository [`Id`] is defined as a path without a leading `/`
/// that does not contain any other component than [`path::Component::Normal`]
/// and has a maximum of two components, ending with `.git`.
#[derive(Debug, PartialEq)]
pub struct Id {
    namespace: Option<String>,
    repository: String,
}

impl Id {
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    pub fn repository(&self) -> &str {
        self.repository.as_ref()
    }

    /// Constructs the authority repository [`Id`]
    /// from the current `namespace`:`repository` couple.
    pub fn to_authority(&self) -> Id {
        Self {
            namespace: self.namespace.clone(),
            repository: AUTHORITY_REPOSITORY_NAME.into(),
        }
    }

    /// Checks if the current [`Id`] is the authority's repository path.
    pub fn is_authority(&self) -> bool {
        self.repository == AUTHORITY_REPOSITORY_NAME
    }

    /// Converts the current [`Id`] to a [`PathBuf`], in the `sotrage` path.
    pub fn to_path(&self, storage: &Path) -> PathBuf {
        storage.join(self.to_string())
    }
}

impl FromStr for Id {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Encapsulate in a path and strip any leading `/`
        let path = Path::new(s);
        let path = path.strip_prefix("/").unwrap_or(path);

        let components: Vec<_> = path.components().collect();

        let (namespace, repository) = match components[..] {
            // Enforce the `.git` extension in repository names
            _ if path.extension() != Some(REPOSITORY_NAME_EXT[1..].as_ref()) => {
                return Err(eyre::eyre!(
                    "The repository name must end with the `.git` extension"
                ))
            }
            [path::Component::Normal(repository)] => (None, repository),
            [path::Component::Normal(namespace), path::Component::Normal(repository)] => {
                (Some(namespace), repository)
            }
            _ => return Err(eyre::eyre!("The repository path is misformatted")),
        };

        let namespace = namespace.map(OsStr::to_string_lossy).map(Cow::into_owned);
        let repository = repository.to_string_lossy().into_owned();

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
        f.write_str(&self.repository)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/user/repo.git",
        Id { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("user/repo.git",
        Id { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("//user/repo.git",
        Id { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("/~weirduser/repo.git",
        Id { namespace: Some("~weirduser".into()), repository: "repo.git".into() })]
    #[case("/user/~weirdrepo.git",
        Id { namespace: Some("user".into()), repository: "~weirdrepo.git".into() })]
    #[case("..git",
        Id { namespace: None, repository: "..git".into() })]
    #[case("~/repo.git",
        Id { namespace: Some("~".into()), repository: "repo.git".into() })]
    #[case("/~/repo.git",
        Id { namespace: Some("~".into()), repository: "repo.git".into() })]
    #[case("/./repo.git",
        Id { namespace: None, repository: "repo.git".into() })]
    #[case(AUTHORITY_REPOSITORY_NAME,
            Id { namespace: None, repository: AUTHORITY_REPOSITORY_NAME.into() })]
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
    fn it_denies_sketchy_repositories(#[case] path: &str) {
        let _ = Id::from_str(path).unwrap_err();
    }
}
