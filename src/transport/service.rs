use parse_display::FromStr;

use crate::repository;

#[derive(Debug, FromStr)]
#[display("{} '{path}'", style = "kebab-case")]
pub enum Service {
    GitUploadPack { path: repository::Path },
    GitReceivePack { path: repository::Path },
}

impl Service {
    pub fn path(&self) -> &repository::Path {
        match self {
            Service::GitUploadPack { path } => path,
            Service::GitReceivePack { path } => path,
        }
    }
}
