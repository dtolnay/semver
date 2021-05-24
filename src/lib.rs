#![doc(html_root_url = "https://docs.rs/semver/0.0.0")]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(all(not(feature = "std"), not(no_alloc_crate)), no_std)]
#![cfg_attr(not(no_unsafe_op_in_unsafe_fn_lint), deny(unsafe_op_in_unsafe_fn))]
#![cfg_attr(no_unsafe_op_in_unsafe_fn_lint, allow(unused_unsafe))]
#![cfg_attr(no_str_strip_prefix, allow(unstable_name_collisions))]

#[cfg(not(no_alloc_crate))]
extern crate alloc;

mod backport;
mod display;
mod error;
mod eval;
mod identifier;
mod impls;
mod parse;

#[cfg(feature = "serde")]
mod serde;

use crate::alloc::vec::Vec;
use crate::identifier::Identifier;
use core::str::FromStr;

#[allow(unused_imports)]
use crate::backport::*;

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
#[cfg_attr(not(no_non_exhaustive), non_exhaustive)]
pub enum Op {
    Exact,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    Tilde,
    Caret,
    Wildcard,

    #[cfg(no_non_exhaustive)] // rustc <1.40
    #[doc(hidden)]
    __NonExhaustive,
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
        Version::from_str(text)
    }
}

impl VersionReq {
    #[cfg(not(no_const_vec_new))] // rustc <1.39
    pub const STAR: Self = VersionReq {
        comparators: Vec::new(),
    };

    pub fn parse(text: &str) -> Result<Self, Error> {
        VersionReq::from_str(text)
    }

    pub fn matches(&self, version: &Version) -> bool {
        eval::matches_req(self, version)
    }
}

impl Comparator {
    pub fn parse(text: &str) -> Result<Self, Error> {
        Comparator::from_str(text)
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
        Prerelease::from_str(text)
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
        BuildMetadata::from_str(text)
    }

    pub fn as_str(&self) -> &str {
        self.identifier.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.identifier.is_empty()
    }
}
