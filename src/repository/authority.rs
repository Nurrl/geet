use git2::Oid;
use ssh_key::PublicKey;

use super::{
    entries::{self, Entry, Global, Keychain, Repositories},
    Repository,
};

pub struct GlobalAuthority {
    pub global: Global,
    pub local: LocalAuthority,
}

impl GlobalAuthority {
    pub fn load(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Global::load_or_init(repository, ())?,
            local: LocalAuthority::load(repository, key)?,
        })
    }

    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            global: Global::load_at(repository, reference)?,
            local: LocalAuthority::load_at(repository, reference)?,
        })
    }
}

pub struct LocalAuthority {
    pub keychain: Keychain,
    pub repositories: Repositories,
}

impl LocalAuthority {
    pub fn load(repository: &Repository, key: &PublicKey) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Keychain::load_or_init(repository, key)?,
            repositories: Repositories::load_or_init(repository, ())?,
        })
    }

    pub fn load_at(repository: &Repository, reference: Oid) -> Result<Self, entries::Error> {
        Ok(Self {
            keychain: Keychain::load_at(repository, reference)?,
            repositories: Repositories::load_at(repository, reference)?,
        })
    }
}
