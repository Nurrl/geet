use clap::Parser;
use color_eyre::eyre;

use super::{Error, Params};

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
#[derive(Debug, Parser)]
pub struct PostReceive {
    #[command(flatten)]
    params: Params,
}

impl PostReceive {
    pub fn run(self) -> Result<(), Error<eyre::Error>> {
        println!("Successfully updated refs :: ✓");

        Ok(())
    }
}
