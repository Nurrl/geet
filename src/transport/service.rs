use std::{
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Output, Stdio},
};

use color_eyre::eyre;
use parse_display::{Display, FromStr};
use russh::{server::Msg, Channel};
use tokio::process::Command;

use crate::repository;

/// A representation of the service request received from the git client,
/// parsed from the command sent by git.
#[derive(Debug, FromStr, Display)]
#[display("{} '{repository}'", style = "kebab-case")]
pub enum Service {
    /// Invoked by `git fetch-pack`, learns what objects the other side is missing, and sends them after packing.
    GitUploadPack { repository: repository::Id },

    /// Invoked by `git send-pack` and updates the repository with the information fed from the remote end.
    GitReceivePack { repository: repository::Id },
}

impl Service {
    pub fn repository(&self) -> &repository::Id {
        match self {
            Service::GitUploadPack { repository } => repository,
            Service::GitReceivePack { repository } => repository,
        }
    }

    pub fn access(&self) -> ServiceAccess {
        match self {
            Service::GitUploadPack { .. } => ServiceAccess::Read,
            Service::GitReceivePack { .. } => ServiceAccess::Write,
        }
    }

    pub async fn exec(
        &self,
        envs: impl IntoIterator<Item = (impl AsRef<OsStr>, impl AsRef<OsStr>)>,
        storage: &Path,
        channel: &mut Channel<Msg>,
    ) -> eyre::Result<ExitStatus> {
        let mut child = match self {
            Self::GitUploadPack { repository } => Command::new("git-upload-pack")
                .env_clear()
                .envs(envs)
                .arg("--strict")
                .arg("--timeout=1")
                .arg(repository.to_path(storage))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?,
            Self::GitReceivePack { repository } => Command::new("git-receive-pack")
                .env_clear()
                .envs(envs)
                .arg(repository.to_path(storage))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true)
                .spawn()?,
        };

        let (mut stdout, mut stdin) = (
            child
                .stdout
                .take()
                .expect("Unable to take service's `stdout` handle"),
            child
                .stdin
                .take()
                .expect("Unable to take service's `stdin` handle"),
        );
        let (mut tx, mut rx) = channel.into_io_parts();

        tokio::try_join!(
            async move {
                let res = tokio::io::copy(&mut rx, &mut stdin).await;
                drop(stdin);

                res
            },
            async move {
                let res = tokio::io::copy(&mut stdout, &mut tx).await;
                drop(tx);

                res
            },
        )?;

        let Output { status, stderr, .. } = child.wait_with_output().await?;

        if !stderr.is_empty() {
            tracing::warn!(
                "Service additionnal output (code {}): {}",
                status.code().unwrap_or(i32::MAX),
                String::from_utf8_lossy(&stderr)
            );
        }

        Ok(status)
    }
}

/// A definition of what access the services requires to perform it's action.
#[derive(Debug, PartialEq)]
pub enum ServiceAccess {
    Read,
    Write,
}
