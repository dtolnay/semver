use crate::{BuildMetadata, Comparator, Prerelease, Version, VersionReq};
use std::str::FromStr;

pub struct Error {
    _todo: (),
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for VersionReq {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for Comparator {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for Prerelease {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for BuildMetadata {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}
