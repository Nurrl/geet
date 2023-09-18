use std::path::PathBuf;

use parse_display::FromStr;

#[derive(Debug, FromStr)]
#[display("{} '{repository}'", style = "kebab-case")]
pub enum Service {
    GitUploadPack { repository: PathBuf },
    GitReceivePack { repository: PathBuf },
}
