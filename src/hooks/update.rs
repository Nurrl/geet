use clap::Parser;
use color_eyre::eyre::{self, WrapErr};

use super::{Error, Params};
use crate::repository::{
    authority::{Authority, Namespace, Origin},
    id::Type,
    Repository,
};

/// The update script is very similar to the pre-receive script,
/// except that it’s run once for each branch the pusher is trying to update.
/// If the pusher is trying to push to multiple branches, pre-receive runs only once,
/// whereas update runs once per branch they’re pushing to.
/// Instead of reading from stdin, this script takes three arguments: the name of the reference (branch),
/// the SHA-1 that reference pointed to before the push, and the SHA-1 the user is trying to push.
/// If the update script exits non-zero, only that reference is rejected;
/// other references can still be updated.
///
/// see https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks#_update
#[derive(Debug, Parser)]
pub struct Update {
    #[command(flatten)]
    params: Params,

    /// The reference being currently updated.
    reference: String,
    /// The SHA-1 of the commit pointed by `reference` before updating.
    before: String,
    /// The SHA-1 of the commit pointed by `reference` after updating.
    after: String,
}

impl Update {
    pub fn run(self) -> Result<(), Error<eyre::Error>> {
        let Params { storage, id } = self.params;

        let repository = Repository::open(&storage, &id)?;
        let is_head = self.reference.as_bytes() == repository.head()?.name_bytes();

        match id.as_type() {
            Type::OriginAuthority(_) => {
                match Origin::read_commit(&repository, &self.after)
                    .wrap_err("Authority update failed")
                {
                    Ok(_) => Ok(()),
                    // If repository's head is updated, ensure authority integrity
                    Err(err) if is_head => Err(Error::Err(err)),
                    // If another branch is updated, simply spit out a warning
                    Err(err) => Err(Error::Warn(err)),
                }
            }
            Type::NamespaceAuthority(_) => {
                match Namespace::read_commit(&repository, &self.after)
                    .wrap_err("Authority update failed")
                {
                    Ok(_) => Ok(()),
                    // If repository's head is updated, ensure authority integrity
                    Err(err) if is_head => Err(Error::Err(err)),
                    // If another branch is updated, simply spit out a warning
                    Err(err) => Err(Error::Warn(err)),
                }
            }
            Type::Plain(id) => {
                let authority = Repository::open(&storage, &id.to_authority())?;

                let def = if id.namespace().is_none() {
                    Origin::read(&authority)?
                        .repository(id)
                        .expect("The repository is not defined in it's authority repository")
                        .clone()
                } else {
                    Namespace::read(&authority)?
                        .repository(id)
                        .expect("The repository is not defined in it's authority repository")
                        .clone()
                };

                todo!("Perform reference checks");
            }
        }
    }
}
