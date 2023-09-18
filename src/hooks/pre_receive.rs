use clap::Parser;
use color_eyre::eyre;

use super::{Error, Params};

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
    pub fn run(self) -> Result<(), Error<eyre::Error>> {
        Ok(())
    }
}
