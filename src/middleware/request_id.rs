use std::{fmt::Display, ops::Deref};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(crate) struct RequestId(String);

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Deref for RequestId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
