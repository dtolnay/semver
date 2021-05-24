mod parse;

pub use crate::parse::Error;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: String,
    pub build: String,
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
    pub pre: String,
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

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            pre: String::new(),
            build: String::new(),
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
        let _ = version;
        unimplemented!()
    }
}

impl Comparator {
    pub fn parse(text: &str) -> Result<Self, Error> {
        let _ = text;
        unimplemented!()
    }

    pub fn matches(&self, version: &Version) -> bool {
        let _ = version;
        unimplemented!()
    }
}
