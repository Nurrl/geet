use parse_display::FromStr;

use crate::repository;

#[derive(Debug, FromStr)]
#[display("{} '{repository}'", style = "kebab-case")]
pub enum Service {
    GitUploadPack { repository: repository::Id },
    GitReceivePack { repository: repository::Id },
}

impl Service {
    pub fn repository(&self) -> &repository::Id {
        match self {
            Service::GitUploadPack { repository } => repository,
            Service::GitReceivePack { repository } => repository,
        }
    }

    pub fn command(&self) -> &'static str {
        match self {
            Self::GitUploadPack { .. } => "git-upload-pack",
            Self::GitReceivePack { .. } => "git-receive-pack",
        }
    }
}
