use super::{Base, Error};

/// The standard extension for git repositories.
pub const REPOSITORY_NAME_EXT: &str = ".git";

/// A valid repository name, ending with the [`REPOSITORY_NAME_EXT`] suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Name(pub(crate) Base);

impl From<Base> for Name {
    fn from(value: Base) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for Name {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let identifier = s
            .strip_suffix(REPOSITORY_NAME_EXT)
            .ok_or(Error::MissingExt(REPOSITORY_NAME_EXT))?;

        identifier.parse().map(Self)
    }
}

impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.0, REPOSITORY_NAME_EXT)
    }
}

impl std::ops::Deref for Name {
    type Target = Base;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
