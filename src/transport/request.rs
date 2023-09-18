use std::{collections::HashMap, path::PathBuf};

use color_eyre::eyre::{self, WrapErr};
use russh::{server::Msg, Channel};
use russh_keys::key;

use crate::repository::{Authority, Repository};

use super::Service;

#[derive(Debug)]
pub struct Request {
    key: key::PublicKey,
    storage: PathBuf,
    channel: Channel<Msg>,
    envs: HashMap<String, String>,
}

impl Request {
    pub fn new(key: key::PublicKey, storage: PathBuf, channel: Channel<Msg>) -> Self {
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

        let repository = Repository::open(&self.storage, &service.repository().to_authority())
            .wrap_err("Failed to open the git repository")?;

        if repository.head()?.target().is_none() {
            let authority = Authority::init(service.repository().namespace(), self.key.clone());
            authority
                .store(&repository)
                .wrap_err("Failed to initialize the Authority repository")?;
        }

        let authority = Authority::load(&repository)
            .wrap_err("Failed to load the Authority from the repository")?;

        Ok(())
    }
}
