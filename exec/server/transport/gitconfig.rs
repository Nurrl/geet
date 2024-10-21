use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use furrow::DEFAULT_BRANCH;

/// A handle to a `.gitconfig` file for our remote.
#[derive(Debug)]
pub struct GitConfig {
    path: PathBuf,
}

impl GitConfig {
    /// The name of the file being populated in the remote's storage.
    pub const PATH: &'static str = ".gitconfig";

    pub fn new(storage: &Path) -> Self {
        let path = storage.join(Self::PATH);

        Self { path }
    }

    /// Populate the gitconfig file with the following settings.
    ///
    /// - `init.defaultBranch`: [`DEFAULT_BRANCH`]
    ///   Makes `git` aware that we have a different branch set-up as default branch.
    ///
    /// - `receive.keepAlive`: `3`
    ///   Send a keep-alive every `n` seconds, to prevent the server timing out.
    ///
    /// - `receive.fsckObject`: `true`
    ///   Makes `git-receive-pack` check all received objects for errors or security issues.
    ///
    /// - `receive.denyDeleteCurrent`: `false`
    ///   Allows the client to delete the `HEAD` branch.
    ///
    pub fn populate(&self) -> Result<(), git2::Error> {
        let mut config = git2::Config::open(&self.path)?;

        tracing::debug!(
            "Populating our own `{}` file at `{}`",
            Self::PATH,
            self.path.display()
        );

        config.set_str("init.defaultBranch", DEFAULT_BRANCH)?;

        config.set_i32("receive.keepAlive", 3)?;
        config.set_bool("receive.fsckObjects", true)?;
        config.set_str("receive.denyDeleteCurrent", "ignore")?;

        Ok(())
    }

    /// Setup the subcommand required environment variables to use the [`GitConfig`].
    pub fn env(&self, envs: &mut HashMap<String, String>) {
        envs.insert(
            "GIT_CONFIG_GLOBAL".into(),
            self.path.to_string_lossy().into(),
        );
    }
}
