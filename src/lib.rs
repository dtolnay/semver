#![deny(unsafe_op_in_unsafe_fn)]

mod display;
mod eval;
mod identifier;
mod impls;
mod parse;

use crate::identifier::Identifier;

pub use crate::parse::Error;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Prerelease,
    pub build: BuildMetadata,
}

#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VersionReq {
    pub comparators: Vec<Comparator>,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Comparator {
    pub op: Op,
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre: Prerelease,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Op {
    Exact,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Tilde,
    Caret,
    Wildcard,
}

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct Prerelease {
    identifier: Identifier,
}

#[derive(Default, Clone, Eq, PartialEq, Hash)]
pub struct BuildMetadata {
    identifier: Identifier,
}

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }

    pub fn parse(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }
}

impl VersionReq {
    pub const STAR: Self = VersionReq {
        comparators: Vec::new(),
    };

    pub fn parse(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }

    pub fn matches(&self, version: &Version) -> bool {
        eval::matches_req(self, version)
    }
}

impl Comparator {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }

    pub fn matches(&self, version: &Version) -> bool {
        eval::matches_comparator(self, version)
    }
}

impl Prerelease {
    pub const EMPTY: Self = Prerelease {
        identifier: Identifier::empty(),
    };

    pub fn new(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }

    pub fn as_str(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.identifier.is_empty()
    }
}

impl BuildMetadata {
    pub const EMPTY: Self = BuildMetadata {
        identifier: Identifier::empty(),
    };

    pub fn new(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }

    pub fn as_str(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.identifier.is_empty()
    }
}
