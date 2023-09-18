use std::{net::SocketAddr, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub address: SocketAddr,
    pub keys: Option<Vec<PathBuf>>,
    pub banner: Option<String>,
    pub storage: PathBuf,
}
