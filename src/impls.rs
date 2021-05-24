use crate::{BuildMetadata, Prerelease};
use std::ops::Deref;

impl Deref for Prerelease {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.identifier.as_str()
    }
}

impl Deref for BuildMetadata {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.identifier.as_str()
    }
}
