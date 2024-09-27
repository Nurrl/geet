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
        authority::{GlobalAuthority, LocalAuthority},
        entries::{RegistrationPolicy, Visibility},
        id::Kind,
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

        // Open the global authority repository.
        let global = Repository::open(self.storage, &Id::global_authority())
            .or_else(|_| Repository::init(self.storage, &Id::global_authority()))?;

        // Load or init the global authority from the repository.
        let authority = GlobalAuthority::load(&global, self.key)?;

        // Automatically create the local authority repository if self-registration
        // is allowed or the requester is from the global authority keychain.
        if service.target().kind() == Kind::LocalAuthority
            && (authority.global.registration == RegistrationPolicy::Allow
                || authority.local.keychain.contains(self.key))
        {
            Repository::open(self.storage, service.target())
                .or_else(|_| Repository::init(self.storage, service.target()))?;
        }

        // Load or init the target authority from the repository.
        let authority = match service.target().kind() {
            Kind::GlobalAuthority => authority.local,
            _ => {
                let repository = Repository::open(self.storage, &service.target().to_authority())?;
                LocalAuthority::load(&repository, self.key)?
            }
        };

        let allowed = if service.target().is_authority() {
            authority.keychain.contains(self.key)
        } else {
            let repository = authority
                .repositories
                .repositories
                .get(service.target().repository())
                .ok_or_else(|| {
                    eyre::eyre!("Missing repository definition for `{}`", service.target())
                })?;

            let allowed = match repository.visibility {
                Visibility::Private => authority.keychain.contains(self.key),
                Visibility::Public => {
                    service.access() == ServiceAccess::Read || authority.keychain.contains(self.key)
                }
                Visibility::Archive => service.access() == ServiceAccess::Read,
            };

            if allowed {
                // Create the target repository if non-existant.
                Repository::open(self.storage, service.target())
                    .or_else(|_| Repository::init(self.storage, service.target()))?;
            }

            allowed
        };

        if allowed {
            // Install our server-side hooks and inject env variables
            Hooks::install(self.storage, service.target())?;
            Hooks::env(&mut envs, self.storage, service.target());

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
            request.accept().await?;

            Err(eyre::eyre!("The access to the repository has been denied"))
        }
    }
}
