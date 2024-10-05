use std::{
    ffi::OsStr,
    path::Path,
    process::{ExitStatus, Output, Stdio},
};

use assh::{side::Side, Pipe};
use assh_connect::channel::{request::Request, Channel};
use async_compat::CompatExt;
use color_eyre::eyre;
use futures::{AsyncReadExt, AsyncWriteExt};
// TODO: Remove parse_display to enable bubbling up the parse errors.
use parse_display::{Display, FromStr};
use tokio::process::Command;

use crate::repository;

/// A definition of what access the services requires to perform it's action.
#[derive(Debug, PartialEq)]
pub enum ServiceAccess {
    Read,
    Write,
}

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
    pub fn target(&self) -> &repository::Id {
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
        channel: &Channel<'_, impl Pipe, impl Side>,
        request: Request<'_, impl Pipe, impl Side>,
    ) -> eyre::Result<ExitStatus> {
        let mut child = match self {
            Self::GitUploadPack { repository } => Command::new("git-upload-pack")
                .env_clear()
                .envs(envs)
                .arg("--strict")
                .arg("--timeout=3")
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

        let (mut stdin, mut stdout) = (
            child.stdin.take().map(CompatExt::compat),
            child
                .stdout
                .take()
                .expect("Unable to take service's `stdout` handle")
                .compat(),
        );

        let (mut reader, mut writer) = (channel.as_reader(), channel.as_writer());
        request.accept().await?;

        let (status, _, _) = tokio::try_join!(
            // Wait for `child process` to exit.
            async move {
                let Output { status, stderr, .. } = child.wait_with_output().await?;

                if !stderr.is_empty() {
                    tracing::warn!(
                        "Service additionnal output (code {}): {}",
                        status.code().unwrap_or(i32::MAX),
                        String::from_utf8_lossy(&stderr)
                    );
                }

                Ok::<_, eyre::Error>(status)
            },
            // Process `peer -> child process` communication.
            async move {
                let mut buf = [0u8; 4096 * 8];

                loop {
                    let n = reader.read(&mut buf[..]).await?;
                    if n == 0 {
                        drop(stdin.take());

                        break Ok(());
                    }

                    if let Some(ref mut stdin) = stdin {
                        stdin.write_all(&buf[..n]).await?;
                        stdin.flush().await?;
                    }
                }
            },
            // Process `child process -> peer` communication.
            async move {
                let mut buf = [0u8; 4096 * 8];

                loop {
                    let n = stdout.read(&mut buf[..]).await?;
                    if n == 0 {
                        break Ok(());
                    }

                    writer.write_all(&buf[..n]).await?;
                    writer.flush().await?;
                }
            }
        )?;

        Ok(status)
    }
}
