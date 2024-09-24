use core::str;
use std::{collections::HashMap, path::Path};

use assh::{
    side::{server::Server, Side},
    Pipe,
};
use assh_auth::handler::publickey::PublicKey;
use assh_connect::channel::{
    request::{ChannelRequestContext, Request},
    Channel,
};
use color_eyre::eyre::{self, WrapErr};
use futures::TryStreamExt;

use crate::{
    hooks::Hooks,
    repository::{
        id::Type,
        source::{Namespace, Origin, Source, Visibility},
        Id, Repository,
    },
    server::Socket,
    transport::service::ServiceAccess,
};

use super::{GitConfig, Service};

/// A tunnel a request is operated in,
/// this handles messages from a `session` type [`Channel`].
pub struct Tunnel<'f> {
    storage: &'f Path,
    gitconfig: &'f GitConfig,

    channel: Channel<'f, Socket, Server>,
    key: &'f PublicKey,
}

impl<'f> Tunnel<'f> {
    pub fn new(
        storage: &'f Path,
        gitconfig: &'f GitConfig,
        channel: Channel<'f, Socket, Server>,
        key: &'f PublicKey,
    ) -> Self {
        Self {
            storage,
            gitconfig,
            channel,
            key,
        }
    }

    pub async fn spin(self) -> eyre::Result<()> {
        let mut requests = self.channel.requests();
        let mut envs = HashMap::new();

        loop {
            let Some(request) = requests.try_next().await? else {
                break;
            };

            match request.cx() {
                ChannelRequestContext::Env { name, value } => {
                    let name = String::from_utf8(name.to_vec())
                        .wrap_err("Received a non-utf8 environment variable name")?;

                    match name.as_ref() {
                        // Restrict the environment variables to theses
                        "GIT_PROTOCOL" => {
                            let value = String::from_utf8(value.to_vec())
                                .wrap_err("Received a non-utf8 environment variable value")?;

                            request.accept().await?;

                            tracing::trace!("Storing environment variable `{name}={value}`");

                            envs.insert(name, value);
                        }
                        _ => {
                            tracing::trace!("Ignored illegal environment variable `{name}`")
                        }
                    }
                }
                ChannelRequestContext::Exec { command, .. } => {
                    drop(requests);

                    let command =
                        str::from_utf8(command).wrap_err("Received a non-utf8 service request")?;
                    let service = command
                        .parse()
                        .wrap_err("Received an illegal service request")
                        .wrap_err_with(|| command.to_string())?;

                    if let Err(err) = self.exec(service, envs, request).await {
                        tracing::warn!("Unable to process service request: {err:#}")
                    }

                    break;
                }
                msg => tracing::trace!("Received an unhandled message: {:?}", msg),
            }
        }

        Ok(())
    }

    /// Process the service request from the requested service
    /// and the acquired context.
    async fn exec(
        &self,
        service: Service,
        mut envs: HashMap<String, String>,
        request: Request<'_, impl Pipe, impl Side>,
    ) -> eyre::Result<()> {
        tracing::info!("Received new service request: {service}");

        // Open the `origin` repository or create it if non-existant.
        let repository = Repository::open(self.storage, &Id::origin())
            .or_else(|_| Repository::init(self.storage, &Id::origin()))?;

        // Load the source or initialize it.
        let origin = Origin::read(&repository).or_else(|_| {
            let source = Origin::init(self.key.clone());

            source
                .commit(&repository, "Source repository initialization")
                .map(|_| source)
        })?;

        let allowed = match service.repository().as_type() {
            Type::OriginSource(_) => origin.has_key(self.key),
            Type::NamespaceSource(id) => {
                let namespace = if origin.allow_registration()
                    || (!origin.allow_registration() && origin.has_key(self.key))
                {
                    // Auto-create and initialize the namespace source repository if:
                    // - The auto-registration is enabled
                    // - The auto-registration is disabled, but the user is owner on the origin repository

                    let repository = Repository::open(self.storage, id)
                        .or_else(|_| Repository::init(self.storage, id))?;

                    Namespace::read(&repository).or_else(|_| {
                        let source = Namespace::init(self.key.clone());

                        source
                            .commit(&repository, "Source repository initialization")
                            .map(|_| source)
                    })?
                } else {
                    Namespace::read(&Repository::open(self.storage, id)?)?
                };

                namespace.has_key(self.key)
            }
            Type::Plain(id) => {
                let source = Namespace::read(&Repository::open(self.storage, &id.to_source())?)?;

                let config = source
                    .repository(id)
                    .ok_or_else(|| eyre::eyre!("Missing repository definition for `{id}`"))?;

                let allowed = match config.visibility {
                    Visibility::Private => source.has_key(self.key),
                    Visibility::Public => {
                        service.access() == ServiceAccess::Read || source.has_key(self.key)
                    }
                    Visibility::Archive => service.access() == ServiceAccess::Read,
                };

                if allowed {
                    // Create the repository if non-existant
                    Repository::open(self.storage, id)
                        .or_else(|_| Repository::init(self.storage, id))?;
                }

                allowed
            }
        };

        if allowed {
            // Install our server-side hooks and inject env variables
            Hooks::install(self.storage, service.repository())?;
            Hooks::env(&mut envs, self.storage, service.repository());

            // Install our own `.gitconfig`
            self.gitconfig.env(&mut envs);

            // Execute the git service
            if let Ok(status) = service
                .exec(&envs, self.storage, &self.channel, request)
                .await
                .wrap_err("Service request transfer failed")
            {
                self.channel
                    .request(ChannelRequestContext::ExitStatus {
                        code: status.code().unwrap_or(1) as u32,
                    })
                    .await?;

                tracing::info!("Service request completed: {service}, {status}");
            }

            Ok(())
        } else {
            Err(eyre::eyre!("Unauthorized access to the repository"))
        }
    }
}
