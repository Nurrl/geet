use thiserror::Error;

#[cfg(doc)]
use super::{Id, Name, Base};

/// An error that can happen during the parsing and usage
/// of [`Id`]s, [`Name`]s or [`Base`]s.
#[derive(Debug, Error)]
pub enum Error {
    #[error("The name is either empty or too long")]
    IllegalSize,

    #[error("A name cannot start or end with `.`")]
    IllegalDot,

    #[error("A name may only contain [a-zA-Z0-9-_?.]")]
    IllegalFormat,

    #[error("The path must comply with the following format: `[<namespace>/]<name>.git`")]
    MisformattedPath,

    #[error("The repository name must include the `{0}` extension")]
    MissingExt(&'static str),
}
