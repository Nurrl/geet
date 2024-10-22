//! Definitions of the different kinds of _authority repositories_.

use git2::Oid;
use ssh_key::PublicKey;

use super::{
    entries::{self, Entry},
    Repository,
};

/// Authority repository _entries_ in the _global_ namespace.
pub struct Global {
    /// Global entries for server-wide configuration.
    pub global: entries::Global,

    /// Local entries for the namespace.
    pub local: Local,
}

impl Global {
    /// Load the entries from the `repository` or init them from the provided arguments.
    pub fn load_or_init(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Entry::load_or_init(repository, ())?,
            local: Local::load_or_init(repository, key)?,
        })
    }

    /// Load the entries from the `repository` at the provided `reference`.
    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Entry::load_at(repository, reference)?,
            local: Local::load_at(repository, reference)?,
        })
    }
}

/// Authority repository _entries_ in any other namespace.
pub struct Local {
    /// Keychain entry for the namespace.
    pub keychain: entries::Keychain,

    /// Repositories entry for the namespace.
    pub repositories: entries::Repositories,
}

impl Local {
    /// Load the entries from the `repository` or init them from the provided arguments.
    pub fn load_or_init(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Entry::load_or_init(repository, key)?,
            repositories: Entry::load_or_init(repository, ())?,
        })
    }

    /// Load the entries from the `repository` at the provided `reference`.
    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Entry::load_at(repository, reference)?,
            repositories: Entry::load_at(repository, reference)?,
        })
    }
}
