use thiserror::Error;

use super::base::AUTHORIZED_NAMES;
#[cfg(doc)]
use super::{Base, Id, Name};

/// An error that can happen during the parsing and usage
/// of [`Id`]s, [`Name`]s or [`Base`]s.
#[derive(Debug, Error)]
pub enum Error {
    #[error("A basename may not be empty or longer than 255 characters")]
    IllegalSize,

    #[error("A basename may not start or end with `.`")]
    IllegalDot,

    #[error("A basename may only match `{}`", *AUTHORIZED_NAMES)]
    IllegalFormat,

    #[error("A basename may not end in `.git`")]
    IllegalExtension,

    #[error("The path must comply with the following format: `[<namespace>/]<name>.git`")]
    MisformattedPath,

    #[error("The name must include the `{0}` extension")]
    MissingExt(&'static str),
}
