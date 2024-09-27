use std::io;

use clap::Parser;
use futures::{io::AllowStdIo, TryStreamExt};

use super::{Error, Params, Ref, RefUpdate};
use crate::repository::{
    authority::{GlobalAuthority, LocalAuthority},
    entries::{Entry, Repositories},
    id::Kind,
    Repository,
};

/// The first script to run when handling a push from a client is pre-receive.
/// It takes a list of references that are being pushed from stdin;
/// if it exits non-zero, none of them are accepted.
/// You can use this hook to do things like make sure none of the updated references are non-fast-forwards,
/// or to do access control for all the refs and files they’re modifying with the push.
///
/// see https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks#_pre_receive
#[derive(Debug, Parser)]
pub struct PreReceive {
    #[command(flatten)]
    params: Params,
}

impl PreReceive {
    pub async fn run(&self) -> Result<(), Error> {
        RefUpdate::from_io(AllowStdIo::new(io::stdin()))
            .try_for_each(|refupdate| self.receive(refupdate))
            .await
    }

    async fn receive(&self, update: RefUpdate) -> Result<(), Error> {
        let Params { storage, id } = &self.params;

        let repository = Repository::open_from_hook(storage, id)?;

        let is_ff = update.is_ff(&repository)?;
        let is_head = update.is_head(&repository)?;
        let is_delete = update.is_delete();

        match id.kind() {
            Kind::GlobalAuthority | Kind::LocalAuthority => {
                if is_delete {
                    return if is_head {
                        Err(Error::DeleteRef(update.refname))
                    } else {
                        // If we allow delete, don't check anything else
                        Ok(())
                    };
                }

                if !is_ff && is_head {
                    return Err(Error::NonFastForward(update.refname));
                }

                // Verify that entries in the repository are correctly
                // formatted before allowing the push.
                let res = if id.namespace().is_none() {
                    GlobalAuthority::load_at(&repository, update.newrev)
                        .map(|_| ())
                        .map_err(Error::from)
                } else {
                    LocalAuthority::load_at(&repository, update.newrev)
                        .map(|_| ())
                        .map_err(Error::from)
                };

                if !is_head {
                    res.map_err(Error::into_hint)
                } else {
                    res
                }
            }
            Kind::Normal => {
                let repositories =
                    Repositories::load(&Repository::open(storage, &id.to_authority())?)?;
                let repository = repositories
                    .repositories
                    .get(id.repository())
                    .expect("The repository is not defined in it's authority repository");

                match (&update.refname, &repository.branches, &repository.tags) {
                    (Ref::Branch(name), Some(regex), _) if !regex.is_match(name) => {
                        return Err(Error::IllegalRefName(name.into(), regex.clone()))?
                    }
                    (Ref::Tag(name), _, Some(regex)) if !regex.is_match(name) => {
                        return Err(Error::IllegalRefName(name.into(), regex.clone()))?
                    }
                    _ => (),
                }

                let refconfig = match &update.refname {
                    Ref::Branch(name) => repository.branch.get(name).cloned().unwrap_or_default(),
                    Ref::Tag(_) => Default::default(),
                };

                if !refconfig.allow_delete && is_delete {
                    return Err(Error::DeleteRef(update.refname));
                }

                if !refconfig.allow_force && !is_ff {
                    return Err(Error::NonFastForward(update.refname));
                }

                Ok(())
            }
        }
    }
}
