use std::{collections::HashMap, path::PathBuf, sync::Arc};

use color_eyre::eyre::{self, WrapErr};
use russh::{
    server::{Handle, Msg},
    Channel, ChannelMsg,
};
use tokio::task::JoinHandle;
use tracing::Instrument;

use crate::{
    hooks::Hooks,
    repository::{
        id::Type,
        source::{Namespace, Origin, Source, Visibility},
        Id, Repository,
    },
    transport::service::ServiceAccess,
};

use super::{GitConfig, PubKey, Service};

/// A tunnel a request is operated in,
/// this handles messages from a `session` type [`Channel`].
pub struct Tunnel {
    storage: PathBuf,
    gitconfig: Arc<GitConfig>,

    key: PubKey,
    channel: Channel<Msg>,
    session: Handle,

    envs: HashMap<String, String>,
}

impl Tunnel {
    pub fn new(
        storage: PathBuf,
        gitconfig: Arc<GitConfig>,
        key: PubKey,
        channel: Channel<Msg>,
        session: Handle,
    ) -> Self {
        Self {
            storage,
            gitconfig,
            key,
            channel,
            session,
            envs: Default::default(),
        }
    }

    pub fn spawn(mut self) -> JoinHandle<()> {
        let span = tracing::span!(
            tracing::Level::INFO,
            "service-request",
            key = %self.key,
            channel = %self.channel.id(),
        );

        tokio::spawn(
            async move {
                loop {
                    match self.channel.wait().await {
                        Some(ChannelMsg::SetEnv {
                            variable_name,
                            variable_value,
                            ..
                        }) => self.set_env(variable_name, variable_value).await,
                        Some(ChannelMsg::Exec { command, .. }) => {
                            if let Err(err) = self.exec(command).await {
                                tracing::warn!("Unable to proccess service request: {err:#}",)
                            }

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
            }
            .instrument(span),
        )
    }

    /// Push a new environment variable to the service request,
    /// the environment will only be saved if deemed safe and necessary.
    async fn set_env(&mut self, name: String, value: String) {
        match name.as_str() {
            // Restrict the environment variables to theses
            "GIT_PROTOCOL" => {
                tracing::trace!("Stored environment variable `{name}={value}`");

                self.envs.insert(name, value);
            }
            _ => tracing::trace!("Ignored illegal environment variable `{name}={value}`"),
        }

        let _ = self.session.channel_success(self.channel.id()).await;
    }

    /// Process the service request from the requested service
    /// and the acquired context.
    async fn exec(&mut self, command: Vec<u8>) -> eyre::Result<()> {
        let command = String::from_utf8(command).wrap_err("Received a non-utf8 service request")?;
        let service: Service = command
            .parse()
            .wrap_err("Received an illegal service request")
            .wrap_err(command)?;

        tracing::info!("Received new service request: {service}",);

        // Open the `origin` repository or create it if non-existant.
        let repository = Repository::open(&self.storage, &Id::origin())
            .or_else(|_| Repository::init(&self.storage, &Id::origin()))?;

        // Load the source or initialize it.
        let origin = Origin::read(&repository).or_else(|_| {
            let source = Origin::init(self.key.clone());

            source
                .commit(&repository, "Source repository initialization")
                .map(|_| source)
        })?;

        let allowed = match service.repository().as_type() {
            Type::OriginSource(_) => origin.has_key(&self.key),
            Type::NamespaceSource(id) => {
                let namespace = if origin.allow_registration()
                    || (!origin.allow_registration() && origin.has_key(&self.key))
                {
                    // Auto-create and initialize the namespace source repository if:
                    // - The auto-registration is enabled
                    // - The auto-registration is disabled, but the user is owner on the origin repository

                    let repository = Repository::open(&self.storage, id)
                        .or_else(|_| Repository::init(&self.storage, id))?;

                    Namespace::read(&repository).or_else(|_| {
                        let source = Namespace::init(self.key.clone());

                        source
                            .commit(&repository, "Source repository initialization")
                            .map(|_| source)
                    })?
                } else {
                    Namespace::read(&Repository::open(&self.storage, id)?)?
                };

                namespace.has_key(&self.key)
            }
            Type::Plain(id) => {
                let source = Namespace::read(&Repository::open(&self.storage, &id.to_source())?)?;

                let config = source
                    .repository(id)
                    .ok_or_else(|| eyre::eyre!("Missing repository definition for `{id}`"))?;

                let allowed = match config.visibility {
                    Visibility::Private => source.has_key(&self.key),
                    Visibility::Public => {
                        service.access() == ServiceAccess::Read || source.has_key(&self.key)
                    }
                    Visibility::Archive => service.access() == ServiceAccess::Read,
                };

                if allowed {
                    // Create the repository if non-existant
                    Repository::open(&self.storage, id)
                        .or_else(|_| Repository::init(&self.storage, id))?;
                }

                allowed
            }
        };

        if allowed {
            // Install our server-side hooks and inject env variables
            Hooks::install(&self.storage, service.repository())?;
            Hooks::env(&mut self.envs, &self.storage, service.repository());

            // Install our own `.gitconfig`
            self.gitconfig.env(&mut self.envs);

            // Execute the git service
            let id = self.channel.id();
            match service
                .exec(&self.envs, &self.storage, &mut self.channel)
                .await
            {
                Ok(status) => {
                    let _ = self.session.channel_success(id).await;
                    let _ = self
                        .session
                        .exit_status_request(id, status.code().unwrap_or(1) as u32)
                        .await;

                    tracing::info!("Service request completed: {service}, {status}");

                    Ok(())
                }
                Err(err) => {
                    let _ = self.session.channel_failure(id).await;

                    Err(err).wrap_err("Service request transfer failed")
                }
            }
        } else {
            let _ = self.session.channel_failure(self.channel.id()).await;

            Err(eyre::eyre!("Unauthorized access to the repository"))
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        futures::executor::block_on(async {
            if let Err(err) = self.channel.close().await {
                tracing::error!(
                    "Failed to automatically close channel #{}: {err}",
                    self.channel.id()
                );
            }
        });
    }
}
