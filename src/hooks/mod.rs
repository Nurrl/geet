use std::{path::PathBuf, str::FromStr};

use color_eyre::eyre::{self, Context};

use crate::repository::{
    self,
    authority::{self, Authority, Namespace, Origin},
    id::Type,
    Id, Repository,
};

/// The name of the environment variable used to pass the repository id to the hooks.
pub const REPOSITORY_ID_ENV: &str = "GEET_REPOSITORY_ID";

/// The name of the environment variable used to pass the global repositories storage path.
pub const STORAGE_PATH_ENV: &str = "GEET_STORAGE_PATH";

/// The list of available server hooks.
pub const HOOKS: &[&str] = &["pre-receive", "update", "post-receive"];

/// The first script to run when handling a push from a client is pre-receive.
/// It takes a list of references that are being pushed from stdin;
/// if it exits non-zero, none of them are accepted.
/// You can use this hook to do things like make sure none of the updated references are non-fast-forwards,
/// or to do access control for all the refs and files they’re modifying with the push.
///
/// see https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks#_pre_receive
pub fn pre_receive() -> eyre::Result<()> {
    Ok((/* This hook is left unused */))
}

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
pub fn update(reference: String, _before: String, after: String) -> eyre::Result<()> {
    let storage = PathBuf::from(std::env::var(STORAGE_PATH_ENV)?);
    let id = Id::from_str(&std::env::var(REPOSITORY_ID_ENV)?)?;

    let repository = Repository::open(&storage, &id)?;
    let is_head = reference.as_bytes() == repository.head()?.name_bytes();

    match id.as_type() {
        Type::OriginAuthority(_) => {
            // If repository's head is updated, ensure authority integrity
            if is_head {
                Origin::read_commit(&repository, &after).wrap_err("Authority update failed")?;
            }

            Ok(())
        }
        Type::NamespaceAuthority(_) => {
            // If repository's head is updated, ensure authority integrity
            if is_head {
                Namespace::read_commit(&repository, &after).wrap_err("Authority update failed")?;
            }

            Ok(())
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

            Ok(())
        }
    }
}

/// The post-receive hook runs after the entire process is completed
/// and can be used to update other services or notify users.
/// It takes the same stdin data as the pre-receive hook.
/// Examples include emailing a list, notifying a continuous integration server,
/// or updating a ticket-tracking system – you can even parse the commit messages
/// to see if any tickets need to be opened, modified, or closed.
/// This script can’t stop the push process, but the client doesn’t disconnect until it has completed,
/// so be careful if you try to do anything that may take a long time.
///
/// see https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks#_post_receive
pub fn post_receive() -> eyre::Result<()> {
    println!("Successfully updated refs ::");

    Ok(())
}
