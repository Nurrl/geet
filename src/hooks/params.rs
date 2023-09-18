use std::path::PathBuf;

use clap::Parser;

use crate::repository::Id;

/// The name of the environment variable used to pass the repository id to the hooks.
pub const REPOSITORY_ID_ENV: &str = "GEET_REPOSITORY_ID";

/// The name of the environment variable used to pass the global repositories storage path.
pub const STORAGE_PATH_ENV: &str = "GEET_STORAGE_PATH";

#[derive(Debug, Parser)]
pub struct Params {
    #[arg(long, env = STORAGE_PATH_ENV)]
    pub storage: PathBuf,

    #[arg(long, env = REPOSITORY_ID_ENV)]
    pub id: Id,
}
