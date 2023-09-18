use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::{self, WrapErr};
use russh::{
    server::{Handle, Msg},
    Channel, ChannelMsg,
};
use tokio::task::JoinHandle;
use tracing::Instrument;

use crate::repository::{
    authority::{Authority, Namespace, Origin},
    Id, Repository, Type,
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
        let service: Service = String::from_utf8(command)
            .wrap_err("Received a non-utf8 service request")?
            .parse()
            .wrap_err("Received an illegal service request")?;

        tracing::info!("Received new service request: {service:?}",);

        // Open the `origin` repository or create it if non-existant.
        let origin = Repository::open(&self.storage, &Id::origin())
            .or_else(|_| Repository::init(&self.storage, &Id::origin()))?;

        // Load the Authority or initialize it.
        let origin = Origin::read(&origin).or_else(|_| {
            let authority = Origin::init(self.key.clone());

            authority
                .commit(&origin, "Origin repository initialization")
                .map(|_| authority)
        })?;

        let allow = match service.repository().as_type() {
            Type::OriginAuthority(_) => origin.has_key(&self.key),
            Type::NamespaceAuthority(id) => {
                let namespace = match (origin.registration(), Repository::open(&self.storage, id)) {
                    (_, Ok(repository)) => repository,
                    (true, Err(_)) => Repository::init(&self.storage, id)?,
                    (false, err) => err?,
                };

                let namespace = match (origin.registration(), Namespace::read(&namespace)) {
                    (_, Ok(repository)) => repository,
                    (true, Err(_)) => {
                        let authority = Namespace::init(
                            id.namespace().map(ToString::to_string),
                            self.key.clone(),
                        );
                        authority.commit(&namespace, "Namespace repository initialization")?;

                        authority
                    }
                    (false, err) => err?,
                };

                namespace.has_key(&self.key)
            }
            Type::Plain(_id) => unimplemented!(),
        };

        if allow {
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
