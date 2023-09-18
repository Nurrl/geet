use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::{self, WrapErr};
use russh::{
    server::{Handle, Msg},
    Channel, ChannelMsg,
};
use tokio::task::JoinHandle;
use tracing::Instrument;

use crate::{
    repository::{
        authority::{Authority, Namespace, Origin, Visibility},
        id::Type,
        Id, Repository,
    },
    transport::service::ServiceAccess,
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

                if let Err(err) = self.channel.close().await {
                    tracing::error!("Unable to close channel@{}: {err:#}", self.channel.id());
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
                tracing::debug!("Stored environment variable `{name}={value}`");

                self.envs.insert(name, value);
            }
            _ => tracing::debug!("Ignored illegal environment variable `{name}={value}`"),
        }

        let _ = self.session.channel_success(self.channel.id()).await;
    }

    /// Process the service request from the requested service
    /// and the acquired context.
    async fn exec(&mut self, command: Vec<u8>) -> eyre::Result<()> {
        let service: Service = String::from_utf8(command)
            .wrap_err("Received a non-utf8 service request")?
            .parse()
            .wrap_err("Received an illegal service request")?;

        tracing::info!("Received new service request: {service}",);

        // Open the `origin` repository or create it if non-existant.
        let repository = Repository::open(&self.storage, &Id::origin())
            .or_else(|_| Repository::init(&self.storage, &Id::origin()))?;

        // Load the Authority or initialize it.
        let origin = Origin::read(&repository).or_else(|_| {
            let authority = Origin::init(self.key.clone());

            authority
                .commit(&repository, "Origin authority repository initialization")
                .map(|_| authority)
        })?;

        let allow = match service.repository().as_type() {
            Type::OriginAuthority(_) => origin.has_key(&self.key),
            Type::NamespaceAuthority(id) => {
                let namespace = if origin.registration()
                    || (!origin.registration() && origin.has_key(&self.key))
                {
                    // Auto-create and initialize the namespace authority repository if:
                    // - The auto-registration is enabled
                    // - The auto-registration is disabled, but the user is owner on the origin repository

                    let repository = Repository::open(&self.storage, id)
                        .or_else(|_| Repository::init(&self.storage, id))?;

                    Namespace::read(&repository).or_else(|_| {
                        let authority =
                            Namespace::init(id.namespace().map(Into::into), self.key.clone());

                        authority
                            .commit(&repository, "Namespace authority repository initialization")
                            .map(|_| authority)
                    })?
                } else {
                    Namespace::read(&Repository::open(&self.storage, id)?)?
                };

                namespace.has_key(&self.key)
            }
            Type::Plain(id) => {
                let authority =
                    Namespace::read(&Repository::open(&self.storage, &id.to_authority())?)?;

                let def = authority
                    .repository(id)
                    .ok_or_else(|| eyre::eyre!("Missing repository definition for `{id}`"))?;

                let allow = match def.visibility() {
                    Visibility::Private => authority.has_key(&self.key),
                    Visibility::Public => {
                        service.access() == ServiceAccess::Read || authority.has_key(&self.key)
                    }
                    Visibility::Archive => service.access() == ServiceAccess::Read,
                };

                if allow {
                    // Create the repository if non-existant
                    Repository::open(&self.storage, id)
                        .or_else(|_| Repository::init(&self.storage, id))?;
                }

                allow
            }
        };

        if allow {
            // Install our server-side hooks
            Repository::hook(&self.storage, service.repository())?;

            // Execute the git service
            match service
                .exec(&self.envs, &self.storage, &mut self.channel)
                .await
            {
                Ok(status) => {
                    let _ = self.session.channel_success(self.channel.id()).await;
                    let _ = self
                        .session
                        .exit_status_request(self.channel.id(), status.code().unwrap_or(1) as u32)
                        .await;

                    tracing::info!("Service request completed: {service}, {status}");

                    Ok(())
                }
                Err(err) => {
                    let _ = self.session.channel_failure(self.channel.id()).await;

                    Err(err).wrap_err("Service request transfer failed")
                }
            }
        } else {
            let _ = self.session.channel_failure(self.channel.id()).await;

            Err(eyre::eyre!("Unauthorized access to the repository"))
        }
    }
}
