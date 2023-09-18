use std::{collections::HashMap, path::PathBuf, process::Stdio};

use color_eyre::eyre::{self, WrapErr};
use russh::{
    server::{Handle, Msg},
    Channel, ChannelMsg,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    task::JoinHandle,
};
use tracing::Instrument;

use crate::repository::{
    authority::{self, Authority},
    Repository,
};

use super::{Key, Service};

pub struct Request {
    key: Key,
    storage: PathBuf,
    channel: Channel<Msg>,
    session: Handle,
    envs: HashMap<String, String>,
}

impl Request {
    pub fn new(key: Key, storage: PathBuf, channel: Channel<Msg>, session: Handle) -> Self {
        Self {
            key,
            storage,
            channel,
            session,
            envs: Default::default(),
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                match self.channel.wait().await {
                    Some(ChannelMsg::SetEnv {
                        variable_name,
                        variable_value,
                        ..
                    }) => self.set_env(variable_name, variable_value),
                    Some(ChannelMsg::Exec { command, .. }) => {
                        let span = tracing::span!(
                            tracing::Level::INFO,
                            "service-request",
                            key = %self.key,
                            channel = %self.channel.id(),
                        );

                        if let Err(err) = self.exec(command).instrument(span.clone()).await {
                            span.in_scope(|| {
                                tracing::warn!("Unable to proccess service request: {err:#}",)
                            });
                        }

                        self.channel.close().await.expect("Unable to close channel");

                        break;
                    }
                    Some(msg) => tracing::trace!(
                        "Received an unhandled message on channel@{}: {:?}",
                        self.channel.id(),
                        msg
                    ),
                    None => break,
                }
            }
        })
    }

    /// Push a new environment variable to the service request,
    /// the environment will only be saved if deemed safe and necessary.
    fn set_env(&mut self, name: String, value: String) {
        match name.as_str() {
            // Restrict the environment variables to theses
            "GIT_PROTOCOL" => {
                tracing::trace!("Stored environment variable `{name}={value}`");

                self.envs.insert(name, value);
            }
            _ => tracing::trace!("Ignored illegal environment variable `{name}={value}`"),
        }
    }

    /// Process the service request from the requested service
    /// and the acquired context.
    pub async fn exec(&mut self, command: Vec<u8>) -> eyre::Result<()> {
        let service: Service = String::from_utf8(command)
            .wrap_err("Received a non-utf8 service request")?
            .parse()
            .wrap_err("Received an illegal service request")?;

        tracing::info!("Received new service request: {service:?}",);

        let authority = if service.repository().is_authority() {
            let repository = match Repository::open(&self.storage, service.repository().clone()) {
                Ok(repository) => repository,
                // When authority repositories are not yet existing, they're auto-created
                Err(err) if err.code() == git2::ErrorCode::NotFound => {
                    tracing::info!(
                        "Initializing git bare repository '{}', as it was non-existant",
                        service.repository()
                    );

                    Repository::init(&self.storage, service.repository().clone())?
                }
                Err(err) => return Err(err).wrap_err("Failed to open git repository"),
            };

            let authority = match Authority::load(&repository) {
                Ok(authority) => authority,
                Err(authority::Error::Git(err)) if err.code() == git2::ErrorCode::UnbornBranch => {
                    tracing::info!(
                        "Initializing Authority repository '{}', as it was empty",
                        service.repository()
                    );

                    let authority = Authority::init(repository.id().namespace(), self.key.clone());

                    authority.commit(&repository, "Initialize Authority repository")?;

                    authority
                }
                Err(err) => {
                    return Err(err).wrap_err("Failed to load the Authority from the repository")
                }
            };

            authority
        } else {
            let repository = Repository::open(&self.storage, service.repository().to_authority())?;

            Authority::load(&repository)?
        };

        if authority.is_owner(&self.key) {
            let mut child = tokio::process::Command::new(service.command())
                .envs(&self.envs)
                .arg("--strict")
                .arg("--timeout=1")
                .arg(service.repository().to_path(&self.storage))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .kill_on_drop(true)
                .spawn()?;

            let (mut stdin, mut stdout) = (
                child.stdin.take(),
                child
                    .stdout
                    .take()
                    .expect("Unable to take service's `stdout` handle"),
            );

            let mut buf = vec![0u8; 4096];
            loop {
                tokio::select! {
                    Ok(n) = stdout.read(&mut buf) => {
                        if n > 0 {
                            self.channel.data(&buf[..n]).await?;
                        }
                    }
                    Some(msg) = self.channel.wait() => {
                        tracing::trace!("Received SSH channel message: {msg:?}");

                        if let ChannelMsg::Data { data } = &msg {
                            if let Some(stdin) = &mut stdin {
                                stdin.write_all(data).await?;
                            }
                        }
                        if let ChannelMsg::Eof = &msg {
                            if let Some(stdin) = &mut stdin {
                                stdin.flush().await?;
                            }
                            drop(stdin.take());
                        }
                    }
                    Ok(status) = child.wait() => {
                        let _ = self.session.channel_success(self.channel.id()).await;
                        let _ = self
                            .session
                            .exit_status_request(self.channel.id(), status.code().unwrap_or(1) as u32)
                            .await;

                        break;
                    }
                    else => {
                        let _ = self.session.channel_failure(self.channel.id()).await;

                        break;
                    }
                }
            }

            Ok(())
        } else {
            let _ = self.session.channel_failure(self.channel.id()).await;

            Err(eyre::eyre!("Unauthorized access to the repository"))
        }
    }
}
