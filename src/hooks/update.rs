use clap::Parser;
use color_eyre::eyre;

use super::{Error, Params};

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
    refname: String,
    /// The SHA-1 of the commit pointed by `reference` before updating.
    oldrev: String,
    /// The SHA-1 of the commit pointed by `reference` after updating.
    newrev: String,
}

impl Update {
    pub async fn run(self) -> Result<(), Error<eyre::Error>> {
        Ok(())
    }
}
