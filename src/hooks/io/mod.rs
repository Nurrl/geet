//! All the I/O of the hooks, from `env`, `stdin`, to errors.

use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, Stream, TryStreamExt};
use parse_display::{Display, FromStr};

use crate::repository::{Id, Repository};

mod error;
pub use error::Error;

/// The name of the environment variable used to pass the repository id to the hooks.
pub const REPOSITORY_ID_ENV: &str = "REPOSITORY_ID";

/// The name of the environment variable used to pass the global repositories storage path.
pub const STORAGE_PATH_ENV: &str = "STORAGE_PATH";

/// A structure representing the `env` parameters required by the hooks.
#[derive(Debug, Parser)]
pub struct Params {
    #[arg(long, env = STORAGE_PATH_ENV)]
    pub storage: PathBuf,

    #[arg(long, env = REPOSITORY_ID_ENV)]
    pub id: Id,
}

/// A structure representing a ref update parsed from stdin.
#[derive(Debug, FromStr, Display)]
#[display("{oldrev} {newrev} {refname}")]
pub struct RefUpdate {
    pub oldrev: git2::Oid,
    pub newrev: git2::Oid,
    pub refname: String,
}

impl RefUpdate {
    pub fn from_io(io: impl AsyncRead) -> impl Stream<Item = Result<Self, Error>> {
        BufReader::new(io)
            .lines()
            .err_into::<Error>()
            .and_then(
                |line| async move { RefUpdate::from_str(&line).map_err(Error::RefUpdateParse) },
            )
    }

    pub fn is_ff(&self, repository: &Repository) -> Result<bool, Error> {
        match (self.oldrev.is_zero(), self.newrev.is_zero()) {
            (true, _) => Ok(true),
            (_, true) => Ok(false),
            _ => repository
                .graph_descendant_of(self.newrev, self.oldrev)
                .map_err(Into::into),
        }
    }

    pub fn is_head(&self, repository: &Repository) -> Result<bool, Error> {
        Ok(self.refname
            == repository
                .find_reference("HEAD")?
                .symbolic_target()
                .expect("HEAD is not a symbolic reference"))
    }

    pub fn is_delete(&self) -> bool {
        !self.oldrev.is_zero() && self.newrev.is_zero()
    }
}
