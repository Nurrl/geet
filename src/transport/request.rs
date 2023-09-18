use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::{self, WrapErr};
use russh::{server::Msg, Channel};

use crate::repository::{Authority, Repository};

use super::{Key, Service};

#[derive(Debug)]
pub struct Request {
    key: Key,
    storage: PathBuf,
    channel: Channel<Msg>,
    envs: HashMap<String, String>,
}

impl Request {
    pub fn new(key: Key, storage: PathBuf, channel: Channel<Msg>) -> Self {
        Self {
            key,
            storage,
            channel,
            envs: Default::default(),
        }
    }

    /// Push a new environment variable to the service request,
    /// the environment will only be saved if deemed safe and necessary.
    pub fn push_env(&mut self, name: &str, value: &str) {
        match name {
            // Restrict the environment variables to theses
            "GIT_PROTOCOL" => {
                self.envs.insert(name.into(), value.into());

                tracing::trace!("Stored environment variable `{name}={value}`");
            }
            _ => tracing::trace!("Ignored illegal environment variable `{name}={value}`"),
        }
    }

    /// Process the service request from the requested service
    /// and the acquired context.
    pub async fn process(&mut self, data: &[u8]) -> eyre::Result<()> {
        let service: Service = String::from_utf8(data.to_vec())
            .wrap_err("Received a non-utf8 service request")?
            .parse()
            .wrap_err("Received an illegal service request")?;

        tracing::info!("Received new service request: {service:?}",);

        if service.repository().is_authority() {
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
                Err(err) if err.code() == git2::ErrorCode::UnbornBranch => {
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

            tracing::info!("{authority:?}");
        } else {
            unimplemented!("Non-authority repositories not yet implemented.")
        }

        Ok(())
    }
}
