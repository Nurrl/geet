use thiserror::Error;

use super::AUTHORIZED_BASENAMES;
#[cfg(doc)]
use super::{Base, Id, Name};

/// An error that can happen during the parsing and usage
/// of [`Id`]s, [`Name`]s or [`Base`]s.
#[derive(Debug, Error)]
pub enum Error {
    /// The [`Base`] is either empty or too long.
    #[error("A basename may not be empty or longer than 255 characters")]
    IllegalSize,

    /// The [`Base`] starts with a `.` while it is not authorized.
    #[error("A basename may not start or end with `.`")]
    IllegalDot,

    /// The [`Base`] does not match the [`AUTHORIZED_BASENAMES`] regex.
    #[error("A basename may only match `{}`", *AUTHORIZED_BASENAMES)]
    IllegalFormat,

    /// The [`Base`] does end in `.git` suffix, which is not authorized.
    #[error("A basename may not end in `.git`")]
    IllegalExtension,

    /// The [`Id`] path format is misformatted.
    #[error("The path must comply with the following format: `[<namespace>/]<name>.git`")]
    MisformattedPath,

    /// The [`Id`] misses the required extension.
    #[error("The name must include the `{0}` extension")]
    MissingExt(&'static str),
}
