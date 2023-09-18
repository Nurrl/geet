use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use color_eyre::eyre;
use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, Stream, TryStreamExt};
use parse_display::{Display, FromStr};

use super::Error;
use crate::repository::Id;

/// The name of the environment variable used to pass the repository id to the hooks.
pub const REPOSITORY_ID_ENV: &str = "REPOSITORY_ID";

/// The name of the environment variable used to pass the global repositories storage path.
pub const STORAGE_PATH_ENV: &str = "STORAGE_PATH";

#[derive(Debug, Parser)]
pub struct Params {
    #[arg(long, env = STORAGE_PATH_ENV)]
    pub storage: PathBuf,

    #[arg(long, env = REPOSITORY_ID_ENV)]
    pub id: Id,
}

/// A structure representing a ref update parsed from stdin.
#[derive(FromStr, Display)]
#[display("{oldrev} {newrev} {refname}")]
pub struct RefUpdate {
    pub oldrev: String,
    pub newrev: String,
    pub refname: String,
}

impl RefUpdate {
    pub fn from_io(io: impl AsyncRead) -> impl Stream<Item = Result<Self, Error<eyre::Error>>> {
        BufReader::new(io)
            .lines()
            .err_into::<Error<eyre::Error>>()
            .and_then(|line| async move {
                RefUpdate::from_str(&line)
                    .map_err(Into::into)
                    .map_err(Error::Err)
            })
    }
}
