//! All the I/O of the hooks, from `env`, `stdin`, to errors.

use std::{fmt::Display, path::PathBuf, str::FromStr};

use clap::Parser;
use color_eyre::eyre;
use futures::{io::BufReader, AsyncBufReadExt, AsyncRead, Stream, TryStreamExt};
use parse_display::{Display, FromStr};

use crate::repository::{authority, Id};

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

/// An [`Error`] that can occur while executing a hook.
#[derive(Debug)]
pub enum Error<E> {
    Err(E),
    Warn(E),
}

impl<E: Display + AsRef<dyn std::error::Error>> Error<E> {
    /// Acknowledge the error by outputing to `stdout`
    /// and exiting with the correct exit-code.
    pub fn acknowledge(self) -> ! {
        match self {
            Error::Err(err) => {
                print!("error: {err}");
                if let Some(source) = err.as_ref().source() {
                    print!(": {source}");
                }
                println!();

                std::process::exit(1);
            }
            Error::Warn(err) => {
                print!("warning: {err}");
                if let Some(source) = err.as_ref().source() {
                    print!(": {source}");
                }
                println!();

                std::process::exit(0);
            }
        }
    }
}

impl<E> From<authority::Error> for Error<E>
where
    E: From<authority::Error>,
{
    fn from(value: authority::Error) -> Self {
        Self::Err(value.into())
    }
}

impl<E> From<git2::Error> for Error<E>
where
    E: From<git2::Error>,
{
    fn from(value: git2::Error) -> Self {
        Self::Err(value.into())
    }
}

impl<E> From<std::io::Error> for Error<E>
where
    E: From<std::io::Error>,
{
    fn from(value: std::io::Error) -> Self {
        Self::Err(value.into())
    }
}
