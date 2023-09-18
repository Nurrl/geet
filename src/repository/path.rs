use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{self, Path as StdPath},
    str::FromStr,
};

use color_eyre::eyre;

/// The name of the authority repository in the repository root
/// and in repository namespaces.
pub const AUTHORITY_REPOSITORY_NAME: &str = "~.git";

/// The standard extension for git repositories.
pub const REPOSITORY_NAME_EXT: &str = ".git";

/// A repository [`Path`] is defined as a path without a leading `/`
/// that does not contain any other component than [`path::Component::Normal`]
/// and has a maximum of two components, ending with `.git`.
#[derive(Debug, PartialEq)]
pub struct Path {
    namespace: Option<String>,
    repository: String,
}

impl Path {
    /// Constructs the authority repository [`Path`]
    /// from the current `namespace`:`repository` couple.
    pub fn to_authority_path(&self) -> Path {
        Self {
            namespace: self.namespace.clone(),
            repository: super::AUTHORITY_REPOSITORY_NAME.into(),
        }
    }
}

impl FromStr for Path {
    type Err = eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Encapsulate in a path and strip any leading `/`
        let path = StdPath::new(s);
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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("/user/repo.git",
        Path { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("user/repo.git",
        Path { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("//user/repo.git",
        Path { namespace: Some("user".into()), repository: "repo.git".into() })]
    #[case("/~weirduser/repo.git",
        Path { namespace: Some("~weirduser".into()), repository: "repo.git".into() })]
    #[case("/user/~weirdrepo.git",
        Path { namespace: Some("user".into()), repository: "~weirdrepo.git".into() })]
    #[case("..git",
        Path { namespace: None, repository: "..git".into() })]
    #[case("~/repo.git",
        Path { namespace: Some("~".into()), repository: "repo.git".into() })]
    #[case("/~/repo.git",
        Path { namespace: Some("~".into()), repository: "repo.git".into() })]
    #[case("/./repo.git",
        Path { namespace: None, repository: "repo.git".into() })]
    #[case(crate::repository::NAMESPACE_REPOSITORY_NAME,
            Path { namespace: None, repository: crate::repository::NAMESPACE_REPOSITORY_NAME.into() })]
    fn it_allows_valid_repositories(#[case] path: &str, #[case] expected: Path) {
        let path = Path::from_str(path).expect(path);

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
        let _ = Path::from_str(path).unwrap_err();
    }
}
