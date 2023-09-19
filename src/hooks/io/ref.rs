use std::str::FromStr;

use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, Stream, TryStreamExt};
use parse_display::{Display, FromStr};

use super::Error;
use crate::repository::Repository;

/// An enum differentiating references of type [`Ref::Branch`] and of type [`Ref::Tag`].
#[derive(Debug, FromStr, Display)]
pub enum Ref {
    #[display("refs/heads/{0}")]
    Branch(String),

    #[display("refs/tags/{0}")]
    Tag(String),
}

/// A structure representing a ref update parsed from stdin.
#[derive(Debug, FromStr, Display)]
#[display("{oldrev} {newrev} {refname}")]
pub struct RefUpdate {
    pub oldrev: git2::Oid,
    pub newrev: git2::Oid,
    pub refname: Ref,
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
        Ok(self.refname.to_string()
            == repository
                .find_reference("HEAD")?
                .symbolic_target()
                .expect("HEAD is not a symbolic reference"))
    }

    pub fn is_delete(&self) -> bool {
        !self.oldrev.is_zero() && self.newrev.is_zero()
    }
}
