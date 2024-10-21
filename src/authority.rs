//! Definitions of the different kinds of _authority repositories_.

use git2::Oid;
use ssh_key::PublicKey;

use super::{
    entries::{self, Entry, Global, Keychain, Repositories},
    Repository,
};

/// Authority repository _entries_ in the _global_ namespace.
pub struct GlobalAuthority {
    /// Global entries for server-wide configuration.
    pub global: Global,

    /// Local entries for the namespace.
    pub local: LocalAuthority,
}

impl GlobalAuthority {
    /// Load the entries from the `repository` or init them from the provided arguments.
    pub fn load_or_init(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Global::load_or_init(repository, ())?,
            local: LocalAuthority::load_or_init(repository, key)?,
        })
    }

    /// Load the entries from the `repository` at the provided `reference`.
    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Global::load_at(repository, reference)?,
            local: LocalAuthority::load_at(repository, reference)?,
        })
    }
}

/// Authority repository _entries_ in any other namespace.
pub struct LocalAuthority {
    /// Keychain entry for the namespace.
    pub keychain: Keychain,

    /// Repositories entry for the namespace.
    pub repositories: Repositories,
}

impl LocalAuthority {
    /// Load the entries from the `repository` or init them from the provided arguments.
    pub fn load_or_init(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Keychain::load_or_init(repository, key)?,
            repositories: Repositories::load_or_init(repository, ())?,
        })
    }

    /// Load the entries from the `repository` at the provided `reference`.
    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Keychain::load_at(repository, reference)?,
            repositories: Repositories::load_at(repository, reference)?,
        })
    }
}
