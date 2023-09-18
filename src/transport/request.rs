use std::collections::HashMap;

use color_eyre::eyre;
use russh::{server::Msg, Channel};
use russh_keys::key;

use super::Service;

#[derive(Debug)]
pub struct Request {
    key: key::PublicKey,
    channel: Channel<Msg>,
    envs: HashMap<String, String>,
}

impl Request {
    pub fn new(key: key::PublicKey, channel: Channel<Msg>) -> Self {
        Self {
            key,
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
            .inspect_err(|err| {
                tracing::warn!(
                    "Received a non-utf8 service request from `{}` on channel@{}: {err}",
                    self.key.fingerprint(),
                    self.channel.id()
                )
            })?
            .parse()
            .inspect_err(|err| {
                tracing::warn!(
                    "Received an illegal service request from `{}` on channel@{}: {err}",
                    self.key.fingerprint(),
                    self.channel.id()
                )
            })?;

        tracing::info!(
            "Received service request from `{}` on channel@{}: {service:?}",
            self.key.fingerprint(),
            self.channel.id()
        );

        Ok(())
    }
}
