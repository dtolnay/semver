// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::error::Error;
use std::fmt;
use std::result;

use Version;
use version::Identifier;

use self::VersionComponent::{NumericVersionComponent, WildcardVersionComponent};
use self::Op::{Ex, Gt, GtEq, Lt, LtEq, Tilde, Compatible, Wildcard};
use self::WildcardVersion::{Major, Minor, Patch};
use self::ReqParseError::{
    InvalidVersionRequirement,
    OpAlreadySet,
    InvalidSigil,
    VersionComponentsMustBeNumeric,
    InvalidIdentifier,
    MajorVersionRequired,
    UnimplementedVersionRequirement
};

use parser;

/// A `VersionReq` is a set of version comparator sets; it corresponds to the top-level
/// "version range" in the Npm implementation of SemVer:
///   https://docs.npmjs.com/misc/semver#ranges
#[derive(Clone, PartialEq, Debug)]
pub struct VersionReq {
    /// FIXME: this pub is bad but I'm lazy for now
    pub sets: Vec<VersionSet>,
}

/// `VersionSet` is composed of a set of one or more comparators (predicates). A specific
/// version can then be matched against the range to see if it satisfies the constraints
/// set by the predicates.
#[derive(Clone, PartialEq, Debug)]
pub struct VersionSet {
    predicates: Vec<Predicate>
}

enum VersionComponent {
    NumericVersionComponent(u64),
    WildcardVersionComponent
}

#[derive(Clone, PartialEq, Debug)]
enum WildcardVersion {
    Major,
    Minor,
    Patch
}

#[derive(PartialEq,Clone,Debug)]
enum Op {
    Ex,   // Exact
    Gt,   // Greater than
    GtEq, // Greater than or equal to
    Lt,   // Less than
    LtEq, // Less than or equal to
    Tilde, // e.g. ~1.0.0
    Compatible, // compatible by definition of semver, indicated by ^
    Wildcard(WildcardVersion), // x.y.*, x.*, *
}

#[derive(PartialEq,Clone,Debug)]
struct Predicate {
    op: Op,
    major: u64,
    minor: Option<u64>,
    patch: Option<u64>,
    pre: Vec<Identifier>,
}

/// A `ReqParseError` is returned from methods which parse a string into a `VersionReq`. Each
/// enumeration is one of the possible errors that can occur.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ReqParseError {
    /// The given version requirement is invalid.
    InvalidVersionRequirement,
    /// You have already provided an operation, such as `=`, `~`, or `^`. Only use one.
    OpAlreadySet,
    /// The sigil you have written is not correct.
    InvalidSigil,
    /// All components of a version must be numeric.
    VersionComponentsMustBeNumeric,
    /// There was an error parsing an identifier.
    InvalidIdentifier,
    /// At least a major version is required.
    MajorVersionRequired,
    /// An unimplemented version requirement.
    UnimplementedVersionRequirement,
}

impl fmt::Display for ReqParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl Error for ReqParseError {
    fn description(&self) -> &str {
        match *self {
            InvalidVersionRequirement => "the given version requirement is invalid",
            OpAlreadySet => "you have already provided an operation, such as =, ~, or ^; only use one",
            InvalidSigil => "the sigil you have written is not correct",
            VersionComponentsMustBeNumeric => "version components must be numeric",
            InvalidIdentifier => "invalid identifier",
            MajorVersionRequired => "at least a major version number is required",
            UnimplementedVersionRequirement => "the given version requirement is not implemented, yet",
        }
    }
}

pub type Result<T> = result::Result<T, ReqParseError>;

impl VersionReq {
    /// Primary constructor of a `VersionReq`.
    pub fn parse(input: &str) -> Result<VersionReq> {

        let res = parser::try_parse_req(input.trim().as_bytes());

        match res {
            Ok(a) => Ok(a),
            Err(_) => Err(ReqParseError::OpAlreadySet), // for now
        }
    }

    /// `matches()` checks if the given `Version` satisfies any (1 or more) of its
    /// comparator (ie. predicate) sets.
    pub fn matches(&self, version: &Version) -> bool {
        if self.sets.is_empty() {
            true
        } else {
            self.sets.iter().any(|pred| pred.matches(version))
        }
    }
}

impl VersionSet {
    /// `any()` is a factory method which creates a `VersionSet` with no constraints. In other
    /// words, any version will match against it.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionSet;
    ///
    /// let anything = VersionSet::any();
    /// ```
    pub fn any() -> VersionSet {
        VersionSet { predicates: vec!() }
    }

    /// `parse()` is the main constructor of a `VersionSet`. It turns a string like `"^1.2.3"`
    /// and turns it into a `VersionSet` that matches that particular constraint.
    ///
    /// A `Result` is returned which contains a `ReqParseError` if there was a problem parsing the
    /// `VersionSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionSet;
    ///
    /// let version = VersionSet::parse("=1.2.3");
    /// let version = VersionSet::parse(">1.2.3");
    /// let version = VersionSet::parse("<1.2.3");
    /// let version = VersionSet::parse("~1.2.3");
    /// let version = VersionSet::parse("^1.2.3");
    /// let version = VersionSet::parse("<=1.2.3");
    /// let version = VersionSet::parse(">=1.2.3");
    /// ```
    ///
    /// This example demonstrates error handling, and will panic.
    ///
    /// ```should-panic
    /// use semver::VersionSet;
    ///
    /// let version = match VersionSet::parse("not a version") {
    ///     Ok(version) => version,
    ///     Err(e) => panic!("There was a problem parsing: {}", e),
    /// }
    /// ```
    pub fn parse(_: &str) -> Result<VersionSet> {
        Ok(VersionSet { predicates: Vec::new() })
    }

    /// `exact()` is a factory method which creates a `VersionSet` with one exact constraint.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionSet;
    /// use semver::Version;
    ///
    /// let version = Version { major: 1, minor: 1, patch: 1, pre: vec![], build: vec![] };
    /// let exact = VersionSet::exact(&version);
    /// ```
    pub fn exact(version: &Version) -> VersionSet {
        VersionSet { predicates: vec!(Predicate::exact(version)) }
    }

    /// `matches()` matches a given `Version` against this `VersionSet`.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionSet;
    /// use semver::Version;
    ///
    /// let version = Version { major: 1, minor: 1, patch: 1, pre: vec![], build: vec![] };
    /// let exact = VersionSet::exact(&version);
    ///
    /// assert!(exact.matches(&version));
    /// ```
    pub fn matches(&self, version: &Version) -> bool {
        self.predicates.iter().all(|p| p.matches(version)) &&
            self.predicates.iter().any(|p| p.pre_tag_is_compatible(version))
    }
}

impl Predicate {
    fn exact(version: &Version) -> Predicate {
        Predicate {
            op: Ex,
            major: version.major,
            minor: Some(version.minor),
            patch: Some(version.patch),
            pre: version.pre.clone(),
        }
    }

    /// `matches()` takes a `Version` and determines if it matches this particular `Predicate`.
    pub fn matches(&self, ver: &Version) -> bool {
        match self.op {
            Ex => self.is_exact(ver),
            Gt => self.is_greater(ver),
            GtEq => self.is_exact(ver) || self.is_greater(ver),
            Lt => !self.is_exact(ver) && !self.is_greater(ver),
            LtEq => !self.is_greater(ver),
            Tilde => self.matches_tilde(ver),
            Compatible => self.is_compatible(ver),
            Wildcard(_) => self.matches_wildcard(ver)
        }
    }

    fn is_exact(&self, ver: &Version) -> bool {
        if self.major != ver.major {
            return false;
        }

        match self.minor {
            Some(minor) => {
                if minor != ver.minor {
                    return false;
                }
            }
            None => return true
        }

        match self.patch {
            Some(patch) => {
                if patch != ver.patch {
                    return false;
                }
            }
            None => return true
        }

        if self.pre != ver.pre {
            return false;
        }

        true
    }

    // https://docs.npmjs.com/misc/semver#prerelease-tags
    fn pre_tag_is_compatible(&self, ver: &Version) -> bool {
        // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it will only be
        // allowed to satisfy comparator sets if at least one comparator with the same [major,
        // minor, patch] tuple also has a prerelease tag.
        !ver.is_prerelease() || (self.major == ver.major &&
                                 self.minor == Some(ver.minor) &&
                                 self.patch == Some(ver.patch) &&
                                 !self.pre.is_empty())
    }

    fn is_greater(&self, ver: &Version) -> bool {
        if self.major != ver.major {
            return ver.major > self.major;
        }

        match self.minor {
            Some(minor) => {
                if minor != ver.minor {
                    return ver.minor > minor
                }
            }
            None => return false
        }

        match self.patch {
            Some(patch) => {
                if patch != ver.patch {
                    return ver.patch > patch
                }
            }
            None => return false
        }

        if !self.pre.is_empty() {
            return ver.pre.is_empty() || ver.pre > self.pre
        }

        false
    }

    // see https://www.npmjs.org/doc/misc/semver.html for behavior
    fn matches_tilde(&self, ver: &Version) -> bool {
        let minor = match self.minor {
            Some(n) => n,
            None => return self.major == ver.major
        };

        match self.patch {
            Some(patch) => {
                self.major == ver.major && minor == ver.minor &&
                    (ver.patch > patch || (ver.patch == patch && self.pre_is_compatible(ver)))
            }
            None => {
                self.major == ver.major && minor == ver.minor
            }
        }
    }

    // see https://www.npmjs.org/doc/misc/semver.html for behavior
    fn is_compatible(&self, ver: &Version) -> bool {
        if self.major != ver.major {
            return false;
        }

        let minor = match self.minor {
            Some(n) => n,
            None => return self.major == ver.major
        };

        match self.patch {
            Some(patch) => if self.major == 0 {
                if minor == 0 {
                    ver.minor == minor && ver.patch == patch && self.pre_is_compatible(ver)
                } else {
                    ver.minor == minor && (ver.patch > patch ||
                                          (ver.patch == patch && self.pre_is_compatible(ver)))
                }
            } else {
                ver.minor > minor ||
                    (ver.minor == minor && (ver.patch > patch ||
                                           (ver.patch == patch && self.pre_is_compatible(ver))))
            },
            None => if self.major == 0 {
                ver.minor == minor
            } else {
                ver.minor >= minor
            }
        }
    }

    fn pre_is_compatible(&self, ver: &Version) -> bool {
        ver.pre.is_empty() || ver.pre >= self.pre
    }

    // see https://www.npmjs.org/doc/misc/semver.html for behavior
    fn matches_wildcard(&self, ver: &Version) -> bool {
        match self.op {
            Wildcard(Major) => true,
            Wildcard(Minor) => self.major == ver.major,
            Wildcard(Patch) => {
                match self.minor {
                    Some(minor) => self.major == ver.major && minor == ver.minor,
                    None => false  // unreachable
                }
            }
            _ => false  // unreachable
        }
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.sets.is_empty() {
            try!(write!(fmt, "*"));
        } else {
            for (i, ref set) in self.sets.iter().enumerate() {
                if i == 0 {
                    try!(write!(fmt, "{}", set));
                } else {
                    try!(write!(fmt, " || {}", set));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for VersionSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.predicates.is_empty() {
            try!(write!(fmt, "*"));
        } else {
            for (i, ref pred) in self.predicates.iter().enumerate() {
                if i == 0 {
                    try!(write!(fmt, "{}", pred));
                } else {
                    try!(write!(fmt, ", {}", pred));
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Wildcard(Major) => try!(write!(fmt, "*")),
            Wildcard(Minor) => try!(write!(fmt, "{}.*", self.major)),
            Wildcard(Patch) => try!(write!(fmt, "{}.{}.*", self.major, self.minor.unwrap())),
            _ => {
                try!(write!(fmt, "{}{}", self.op, self.major));

                match self.minor {
                    Some(v) => try!(write!(fmt, ".{}", v)),
                    None => ()
                }

                match self.patch {
                    Some(v) => try!(write!(fmt, ".{}", v)),
                    None => ()
                }

                if !self.pre.is_empty() {
                    try!(write!(fmt, "-"));
                    for (i, x) in self.pre.iter().enumerate() {
                        if i != 0 { try!(write!(fmt, ".")) };
                        try!(write!(fmt, "{}", x));
                    }
                }
            },
        }

        Ok(())
    }
}

impl fmt::Display for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ex          => try!(write!(fmt, "= ")),
            Gt          => try!(write!(fmt, "> ")),
            GtEq        => try!(write!(fmt, ">= ")),
            Lt          => try!(write!(fmt, "< ")),
            LtEq        => try!(write!(fmt, "<= ")),
            Tilde       => try!(write!(fmt, "~")),
            Compatible  => try!(write!(fmt, "^")),
             // gets handled specially in Predicate::fmt
            Wildcard(_) => try!(write!(fmt, "")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{VersionReq, VersionSet};
    use super::super::version::Version;
    use super::ReqParseError::{
        InvalidVersionRequirement,
        OpAlreadySet,
        InvalidSigil,
        VersionComponentsMustBeNumeric,
        InvalidIdentifier,
        MajorVersionRequired,
    };

    fn req(s: &str) -> VersionReq {
        VersionReq::parse(s).unwrap()
    }

    fn version(s: &str) -> Version {
        match Version::parse(s) {
            Ok(v) => v,
            Err(e) => panic!("`{}` is not a valid version. Reason: {:?}", s, e)
        }
    }

    fn assert_match(req: &VersionReq, vers: &[&str]) {
        for ver in vers.iter() {
            assert!(req.matches(&version(*ver)), "did not match {}", ver);
        }
    }

    fn assert_not_match(req: &VersionReq, vers: &[&str]) {
        for ver in vers.iter() {
            assert!(!req.matches(&version(*ver)), "matched {}", ver);
        }
    }

    #[test]
    fn test_parsing_default() {
        let r = req("1.0.0");

        assert_eq!(r.to_string(), "^1.0.0".to_string());

        assert_match(&r, &["1.0.0", "1.0.1"]);
        assert_not_match(&r, &["0.9.9", "0.10.0", "0.1.0"]);
    }

    #[test]
    fn test_parsing_exact() {
        let r = req("=1.0.0");

        assert!(r.to_string() == "= 1.0.0".to_string());
        assert_eq!(r.to_string(), "= 1.0.0".to_string());

        assert_match(&r, &["1.0.0"]);
        assert_not_match(&r, &["1.0.1", "0.9.9", "0.10.0", "0.1.0", "1.0.0-pre"]);

        let r = req("=0.9.0");

        assert_eq!(r.to_string(), "= 0.9.0".to_string());

        assert_match(&r, &["0.9.0"]);
        assert_not_match(&r, &["0.9.1", "1.9.0", "0.0.9"]);

        let r = req("=0.1.0-beta2.a");

        assert_eq!(r.to_string(), "= 0.1.0-beta2.a".to_string());

        assert_match(&r, &["0.1.0-beta2.a"]);
        assert_not_match(&r, &["0.9.1", "0.1.0", "0.1.1-beta2.a", "0.1.0-beta2"]);
    }

    #[test]
    pub fn test_parsing_greater_than() {
        let r = req(">= 1.0.0");

        assert_eq!(r.to_string(), ">= 1.0.0".to_string());

        assert_match(&r, &["1.0.0", "2.0.0"]);
        assert_not_match(&r, &["0.1.0", "0.0.1", "1.0.0-pre", "2.0.0-pre"]);

        let r = req(">= 2.1.0-alpha2");

        assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha3", "2.1.0", "3.0.0"]);
        assert_not_match(&r, &["2.0.0", "2.1.0-alpha1", "2.0.0-alpha2", "3.0.0-alpha2"]);
    }

    #[test]
    pub fn test_parsing_less_than() {
        let r = req("< 1.0.0");

        assert_eq!(r.to_string(), "< 1.0.0".to_string());

        assert_match(&r, &["0.1.0", "0.0.1"]);
        assert_not_match(&r, &["1.0.0", "1.0.0-beta", "1.0.1", "0.9.9-alpha"]);

        let r = req("<= 2.1.0-alpha2");

        assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha1", "2.0.0", "1.0.0"]);
        assert_not_match(&r, &["2.1.0", "2.2.0-alpha1", "2.0.0-alpha2", "1.0.0-alpha2"]);
    }

    // Test the `VersionSet` matching functionality with a set of comma-separated requirements.
    #[test]
    pub fn test_sets() {
        let r = req("> 0.0.9, <= 2.5.3");
        assert_eq!(r.to_string(), "> 0.0.9, <= 2.5.3".to_string());
        assert_match(&r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_not_match(&r, &["0.0.8", "2.5.4"]);

        let r = req("0.3.0, 0.4.0");
        assert_eq!(r.to_string(), "^0.3.0, ^0.4.0".to_string());
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.4.0"]);

        let r = req("<= 0.2.0, >= 0.5.0");
        assert_eq!(r.to_string(), "<= 0.2.0, >= 0.5.0".to_string());
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.5.1"]);

        let r = req("0.1.0, 0.1.4, 0.1.6");
        assert_eq!(r.to_string(), "^0.1.0, ^0.1.4, ^0.1.6".to_string());
        assert_match(&r, &["0.1.6", "0.1.9"]);
        assert_not_match(&r, &["0.1.0", "0.1.4", "0.2.0"]);

        assert!(VersionSet::parse("> 0.1.0,").is_err());
        assert!(VersionSet::parse("> 0.3.0, ,").is_err());

        let r = req(">=0.5.1-alpha3, <0.6");
        assert_eq!(r.to_string(), ">= 0.5.1-alpha3, < 0.6".to_string());
        assert_match(&r, &["0.5.1-alpha3", "0.5.1-alpha4", "0.5.1-beta", "0.5.1", "0.5.5"]);
        assert_not_match(&r, &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre"]);
        assert_not_match(&r, &["0.6.0", "0.6.0-pre"]);
    }

    // Test the `VersionReq` matching functionality with a set of "||"-separated version sets.
    #[test]
    pub fn test_comparator_sets() {
        // Don't match any of version 1.*
        let r = req(">= 2.0.0 || < 1.0.0");
        assert_eq!(r.to_string(), ">= 2.0.0 || < 1.0.0".to_string());

        assert_match(&r, &["0.9.0", "2.0.0", "2.1.0"]);
        assert_not_match(&r, &["1.0.0", "1.9.9"]);

        assert!(VersionReq::parse("> 1.2.3 ||| < 0.9.8").is_err());
    }

    #[test]
    pub fn test_parsing_tilde() {
        let r = req("~1");
        assert_match(&r, &["1.0.0", "1.0.1", "1.1.1"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "0.0.9"]);

        let r = req("~1.2");
        assert_match(&r, &["1.2.0", "1.2.1"]);
        assert_not_match(&r, &["1.1.1", "1.3.0", "0.0.9"]);

        let r = req("~1.2.2");
        assert_match(&r, &["1.2.2", "1.2.4"]);
        assert_not_match(&r, &["1.2.1", "1.9.0", "1.0.9", "2.0.1", "0.1.3"]);

        let r = req("~1.2.3-beta.2");
        assert_match(&r, &["1.2.3", "1.2.4", "1.2.3-beta.2", "1.2.3-beta.4"]);
        assert_not_match(&r, &["1.3.3", "1.1.4", "1.2.3-beta.1", "1.2.4-beta.2"]);
    }

    #[test]
    pub fn test_parsing_compatible() {
        let r = req("^1");
        assert_match(&r, &["1.1.2", "1.1.0", "1.2.1", "1.0.1"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "0.1.4"]);
        assert_not_match(&r, &["1.0.0-beta1", "0.1.0-alpha", "1.0.1-pre"]);

        let r = req("^1.1");
        assert_match(&r, &["1.1.2", "1.1.0", "1.2.1"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "1.0.1", "0.1.4"]);

        let r = req("^1.1.2");
        assert_match(&r, &["1.1.2", "1.1.4", "1.2.1"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_not_match(&r, &["1.1.2-alpha1", "1.1.3-alpha1", "2.9.0-alpha1"]);

        let r = req("^0.1.2");
        assert_match(&r, &["0.1.2", "0.1.4"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1"]);
        assert_not_match(&r, &["0.1.2-beta", "0.1.3-alpha", "0.2.0-pre"]);

        let r = req("^0.5.1-alpha3");
        assert_match(&r, &["0.5.1-alpha3", "0.5.1-alpha4", "0.5.1-beta", "0.5.1", "0.5.5"]);
        assert_not_match(&r, &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre", "0.6.0"]);

        let r = req("^0.0.2");
        assert_match(&r, &["0.0.2"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.0.1", "0.1.4"]);

        let r = req("^0.0");
        assert_match(&r, &["0.0.2", "0.0.0"]);
        assert_not_match(&r, &["0.9.1", "2.9.0", "1.1.1", "0.1.4"]);

        let r = req("^0");
        assert_match(&r, &["0.9.1", "0.0.2", "0.0.0"]);
        assert_not_match(&r, &["2.9.0", "1.1.1"]);

        let r = req("^1.4.2-beta.5");
        assert_match(&r, &["1.4.2", "1.4.3", "1.4.2-beta.5", "1.4.2-beta.6", "1.4.2-c"]);
        assert_not_match(&r, &["0.9.9", "2.0.0", "1.4.2-alpha", "1.4.2-beta.4", "1.4.3-beta.5"]);
    }

    #[test]
    pub fn test_parsing_wildcard() {
        let r = req("");
        assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_not_match(&r, &[]);
        let r = req("*");
        assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_not_match(&r, &[]);
        let r = req("x");
        assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_not_match(&r, &[]);
        let r = req("X");
        assert_match(&r, &["0.9.1", "2.9.0", "0.0.9", "1.0.1", "1.1.1"]);
        assert_not_match(&r, &[]);

        let r = req("1.*");
        assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
        assert_not_match(&r, &["0.0.9"]);
        let r = req("1.x");
        assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
        assert_not_match(&r, &["0.0.9"]);
        let r = req("1.X");
        assert_match(&r, &["1.2.0", "1.2.1", "1.1.1", "1.3.0"]);
        assert_not_match(&r, &["0.0.9"]);

        let r = req("1.2.*");
        assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
        assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
        let r = req("1.2.x");
        assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
        assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
        let r = req("1.2.X");
        assert_match(&r, &["1.2.0", "1.2.2", "1.2.4"]);
        assert_not_match(&r, &["1.9.0", "1.0.9", "2.0.1", "0.1.3"]);
    }


    #[test]
    pub fn test_parse_errors() {
        assert_eq!(Err(InvalidVersionRequirement), VersionReq::parse("\0"));
        assert_eq!(Err(OpAlreadySet), VersionReq::parse(">= >= 0.0.2"));
        assert_eq!(Err(InvalidSigil), VersionReq::parse(">== 0.0.2"));
        assert_eq!(Err(VersionComponentsMustBeNumeric), VersionReq::parse("a.0.0"));
        assert_eq!(Err(InvalidIdentifier), VersionReq::parse("1.0.0-"));
        assert_eq!(Err(MajorVersionRequired), VersionReq::parse(">="));
    }
}
