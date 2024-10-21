//! All the I/O of the hooks, from `env`, `stdin`, to errors.

mod error;
pub use error::Error;

mod r#ref;
pub use r#ref::{Ref, RefUpdate};

pub(super) mod params;
pub use params::Params;
