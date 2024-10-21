use std::{borrow::Cow, sync::LazyLock};

use parse_display::Display;
use regex::Regex;
use serde_with::{DeserializeFromStr, SerializeDisplay};

use super::Error;

pub static AUTHORIZED_NAMES: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9-_.]*$").expect("The regex was malformed"));

/// A valid base for either a namespace or a repository name.
#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, DeserializeFromStr, SerializeDisplay)]
pub struct Base(pub(crate) Cow<'static, str>);

impl std::str::FromStr for Base {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_ascii_lowercase();

        if s.is_empty() || s.len() > 255 {
            return Err(Error::IllegalSize);
        }

        if s.starts_with('.') || s.ends_with('.') {
            return Err(Error::IllegalDot);
        }

        if s.ends_with(".git") {
            return Err(Error::IllegalExtension);
        }

        if !AUTHORIZED_NAMES.is_match(&s) {
            return Err(Error::IllegalFormat);
        }

        Ok(Self(s.into()))
    }
}

impl std::ops::Deref for Base {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
