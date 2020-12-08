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
use std::mem;
use std::str;

use semver_parser;
use semver_parser::{Compat, RangeSet};
use version::Identifier;
use Version;

#[cfg(feature = "serde")]
use serde::de::{self, Deserialize, Deserializer, Visitor};
#[cfg(feature = "serde")]
use serde::ser::{Serialize, Serializer};

use self::Op::{Ex, Gt, GtEq, Lt, LtEq};
use self::ReqParseError::*;

/// A `VersionReq` is a struct containing a list of ranges that can apply to ranges of version
/// numbers. Matching operations can then be done with the `VersionReq` against a particular
/// version to see if it satisfies some or all of the constraints.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "diesel", derive(AsExpression, FromSqlRow))]
#[cfg_attr(feature = "diesel", sql_type = "diesel::sql_types::Text")]
pub struct VersionReq {
    ranges: Vec<Range>,
    compat: Compat, // defaults to Cargo
}

impl From<semver_parser::RangeSet> for VersionReq {
    fn from(range_set: semver_parser::RangeSet) -> VersionReq {
        VersionReq {
            ranges: range_set.ranges.into_iter().map(From::from).collect(),
            compat: range_set.compat,
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for VersionReq {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize VersionReq as a string.
        serializer.collect_str(self)
    }
}

// TODO: how to implement deserialize with compatibility?
#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for VersionReq {
    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionReqVisitor;

        /// Deserialize `VersionReq` from a string.
        impl<'de> Visitor<'de> for VersionReqVisitor {
            type Value = VersionReq;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a SemVer version requirement as a string")
            }

            fn visit_str<E>(self, v: &str) -> ::std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                VersionReq::parse(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(VersionReqVisitor)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
enum Op {
    Ex,   // Exact
    Gt,   // Greater than
    GtEq, // Greater than or equal to
    Lt,   // Less than
    LtEq, // Less than or equal to
}

impl From<semver_parser::Op> for Op {
    fn from(op: semver_parser::Op) -> Op {
        match op {
            semver_parser::Op::Eq => Op::Ex,
            semver_parser::Op::Gt => Op::Gt,
            semver_parser::Op::Gte => Op::GtEq,
            semver_parser::Op::Lt => Op::Lt,
            semver_parser::Op::Lte => Op::LtEq,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Range {
    predicates: Vec<Predicate>,
    compat: Compat,
}

impl From<semver_parser::Range> for Range {
    fn from(range: semver_parser::Range) -> Range {
        Range {
            predicates: range.comparator_set.into_iter().map(From::from).collect(),
            compat: range.compat,
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
struct Predicate {
    op: Op,
    major: u64,
    minor: u64,
    patch: u64,
    pre: Vec<Identifier>,
}

impl From<semver_parser::Comparator> for Predicate {
    fn from(comparator: semver_parser::Comparator) -> Predicate {
        Predicate {
            op: From::from(comparator.op),
            major: comparator.major,
            minor: comparator.minor,
            patch: comparator.patch,
            pre: comparator.pre.into_iter().map(From::from).collect(),
        }
    }
}

impl From<semver_parser::Identifier> for Identifier {
    fn from(identifier: semver_parser::Identifier) -> Identifier {
        match identifier {
            semver_parser::Identifier::Numeric(n) => Identifier::Numeric(n),
            semver_parser::Identifier::AlphaNumeric(s) => Identifier::AlphaNumeric(s),
        }
    }
}

/// A `ReqParseError` is returned from methods which parse a string into a [`VersionReq`]. Each
/// enumeration is one of the possible errors that can occur.
/// [`VersionReq`]: struct.VersionReq.html
#[derive(Clone, Debug, PartialEq)]
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
    /// This form of requirement is deprecated.
    DeprecatedVersionRequirement(VersionReq),
}

impl fmt::Display for ReqParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            InvalidVersionRequirement => "the given version requirement is invalid",
            OpAlreadySet => {
                "you have already provided an operation, such as =, ~, or ^; only use one"
            }
            InvalidSigil => "the sigil you have written is not correct",
            VersionComponentsMustBeNumeric => "version components must be numeric",
            InvalidIdentifier => "invalid identifier",
            MajorVersionRequired => "at least a major version number is required",
            UnimplementedVersionRequirement => {
                "the given version requirement is not implemented, yet"
            }
            DeprecatedVersionRequirement(_) => "This requirement is deprecated",
        };
        msg.fmt(f)
    }
}

impl Error for ReqParseError {}

impl From<String> for ReqParseError {
    fn from(other: String) -> ReqParseError {
        match &*other {
            "Null is not a valid VersionReq" => ReqParseError::InvalidVersionRequirement,
            "VersionReq did not parse properly." => ReqParseError::OpAlreadySet,
            _ => ReqParseError::InvalidVersionRequirement,
        }
    }
}

impl VersionReq {
    /// `any()` is a factory method which creates a `VersionReq` with no constraints. In other
    /// words, any version will match against it.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionReq;
    ///
    /// let anything = VersionReq::any();
    /// ```
    pub fn any() -> VersionReq {
        VersionReq {
            ranges: vec![],
            compat: Compat::Cargo,
        }
    }

    /// `parse()` is the main constructor of a `VersionReq`. It takes a string like `"^1.2.3"`
    /// and turns it into a `VersionReq` that matches that particular constraint.
    ///
    /// A `Result` is returned which contains a [`ReqParseError`] if there was a problem parsing the
    /// `VersionReq`.
    /// [`ReqParseError`]: enum.ReqParseError.html
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionReq;
    ///
    /// let version = VersionReq::parse("=1.2.3");
    /// let version = VersionReq::parse(">1.2.3");
    /// let version = VersionReq::parse("<1.2.3");
    /// let version = VersionReq::parse("~1.2.3");
    /// let version = VersionReq::parse("^1.2.3");
    /// let version = VersionReq::parse("1.2.3"); // synonym for ^1.2.3
    /// let version = VersionReq::parse("<=1.2.3");
    /// let version = VersionReq::parse(">=1.2.3");
    /// ```
    ///
    /// This example demonstrates error handling, and will panic.
    ///
    /// ```should_panic
    /// use semver::VersionReq;
    ///
    /// let version = match VersionReq::parse("not a version") {
    ///     Ok(version) => version,
    ///     Err(e) => panic!("There was a problem parsing: {}", e),
    /// };
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error variant if the input could not be parsed as a semver requirement.
    ///
    /// Examples of common error causes are as follows:
    ///
    /// * `\0` - an invalid version requirement is used.
    /// * `>= >= 1.2.3` - multiple operations are used. Only use one.
    /// * `>== 1.2.3` - an invalid operation is used.
    /// * `a.0.0` - version components are not numeric.
    /// * `1.2.3-` - an invalid identifier is present.
    /// * `>=` - major version was not specified. At least a major version is required.
    /// * `0.2*` - deprecated requirement syntax. Equivalent would be `0.2.*`.
    ///
    /// You may also encounter an `UnimplementedVersionRequirement` error, which indicates that a
    /// given requirement syntax is not yet implemented in this crate.
    pub fn parse(input: &str) -> Result<VersionReq, ReqParseError> {
        let range_set = input.parse::<RangeSet>();

        if let Ok(v) = range_set {
            return Ok(From::from(v));
        }

        match VersionReq::parse_deprecated(input) {
            Some(v) => Err(ReqParseError::DeprecatedVersionRequirement(v)),
            None => Err(From::from(range_set.err().unwrap())),
        }
    }

    // TODO: better docs for this
    /// `parse_compat()` is like `parse()`, but it takes an extra argument for compatibility with
    /// other semver implementations, and turns that into a `VersionReq` that matches the
    /// particular constraint and compatibility.
    ///
    /// A `Result` is returned which contains a [`ReqParseError`] if there was a problem parsing the
    /// `VersionReq`.
    /// [`ReqParseError`]: enum.ReqParseError.html
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate semver_parser;
    /// use semver::VersionReq;
    /// use semver_parser::Compat;
    ///
    /// # fn main() {
    ///     let cargo_version = VersionReq::parse_compat("1.2.3", Compat::Cargo);
    ///     let npm_version = VersionReq::parse_compat("1.2.3", Compat::Npm);
    /// # }
    /// ```
    pub fn parse_compat(input: &str, compat: Compat) -> Result<VersionReq, ReqParseError> {
        let range_set = RangeSet::parse(input, compat);

        if let Ok(v) = range_set {
            return Ok(From::from(v));
        }

        match VersionReq::parse_deprecated(input) {
            Some(v) => Err(ReqParseError::DeprecatedVersionRequirement(v)),
            None => Err(From::from(range_set.err().unwrap())),
        }
    }

    fn parse_deprecated(version: &str) -> Option<VersionReq> {
        match version {
            ".*" => Some(VersionReq::any()),
            "0.1.0." => Some(VersionReq::parse("0.1.0").unwrap()),
            "0.3.1.3" => Some(VersionReq::parse("0.3.13").unwrap()),
            "0.2*" => Some(VersionReq::parse("0.2.*").unwrap()),
            "*.0" => Some(VersionReq::any()),
            _ => None,
        }
    }

    /// `exact()` is a factory method which creates a `VersionReq` with one exact constraint.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionReq;
    /// use semver::Version;
    ///
    /// let version = Version { major: 1, minor: 1, patch: 1, pre: vec![], build: vec![] };
    /// let exact = VersionReq::exact(&version);
    /// ```
    pub fn exact(version: &Version) -> VersionReq {
        VersionReq {
            ranges: vec![Range {
                predicates: vec![Predicate::exact(version)],
                compat: Compat::Cargo,
            }],
            compat: Compat::Cargo,
        }
    }

    /// `matches()` matches a given [`Version`] against this `VersionReq`.
    /// [`Version`]: struct.Version.html
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::VersionReq;
    /// use semver::Version;
    ///
    /// let version = Version { major: 1, minor: 1, patch: 1, pre: vec![], build: vec![] };
    /// let exact = VersionReq::exact(&version);
    ///
    /// assert!(exact.matches(&version));
    /// ```
    pub fn matches(&self, version: &Version) -> bool {
        // no ranges means anything matches
        if self.ranges.is_empty() {
            return true;
        }

        self.ranges
            .iter()
            .any(|r| r.matches(version) && r.pre_tag_is_compatible(version))
    }

    /// `is_exact()` returns `true` if there is exactly one version which could match this
    /// `VersionReq`. If `false` is returned, it is possible that there may still only be exactly
    /// one version which could match this `VersionReq`. This function is intended do allow
    /// short-circuiting more complex logic where being able to handle only the possibility of a
    /// single exact version may be cheaper.
    ///
    /// # Examples
    ///
    /// ```
    /// use semver::ReqParseError;
    /// use semver::VersionReq;
    ///
    /// fn use_is_exact() -> Result<(), ReqParseError> {
    ///   assert!(VersionReq::parse("=1.0.0")?.is_exact());
    ///   assert!(!VersionReq::parse("=1.0")?.is_exact());
    ///   assert!(!VersionReq::parse(">=1.0.0")?.is_exact());
    ///   Ok(())
    /// }
    ///
    /// use_is_exact().unwrap();
    /// ```
    pub fn is_exact(&self) -> bool {
        if let [range] = self.ranges.as_slice() {
            if let [predicate] = range.predicates.as_slice() {
                return predicate.has_exactly_one_match();
            }
        }

        false
    }

    pub fn union(&mut self, other: &VersionReq) {
        assert_eq!(self.compat, other.compat);
        self.ranges.extend(other.ranges.iter().cloned());
        self.simplify();
    }

    pub fn intersection(&mut self, other: &VersionReq) {
        assert_eq!(self.compat, other.compat);

        let intersection = Vec::with_capacity(self.ranges.len() * other.ranges.len());

        // We allow any range that matches a predicate from self and a predicate from other.
        // It's not important _which_ predicate from each it matters.
        for p1 in mem::replace(&mut self.ranges, intersection) {
            assert_eq!(self.compat, p1.compat);
            for p2 in other.ranges.iter() {
                assert_eq!(self.compat, p2.compat);

                // Allow anything that matches p1 & p2
                self.ranges.push(Range {
                    compat: self.compat,
                    predicates: p1
                        .predicates
                        .iter()
                        .chain(p2.predicates.iter())
                        .cloned()
                        .collect(),
                });
            }
        }
        self.simplify();
    }

    fn simplify(&mut self) {
        let n = self.ranges.len();
        let mut has_empty = false;
        'or: for i in 1..=self.ranges.len() {
            // We need to walk backwards since we might delete some.
            let i = n - i;
            let range = &mut self.ranges[i];

            if range.predicates.is_empty() {
                // We need to preserve an empty range in case it is the only predicate, since in
                // that case the overall VersionReq will never match. Basically, a version
                // requirement of [] and one of [[]] are _not_ the same. The former matches any
                // version, the second matches none.
                has_empty = true;
                continue;
            }

            // Simplifying each &&-ed predicates is fairly simple, since we know that it must
            // produce a single consecutive range. To produce disjoint ranges, || is needed, but
            // within each predicate there is only &&.
            //
            // NOTE: This will no longer hold true if Op::NotEx is added.
            let mut start = Predicate {
                op: Op::GtEq,
                major: 0,
                minor: 0,
                patch: 0,
                pre: Vec::new(),
            };
            let mut end = Predicate {
                op: Op::LtEq,
                major: std::u64::MAX,
                minor: std::u64::MAX,
                patch: std::u64::MAX,
                pre: Vec::new(),
            };

            let mut predicates = range.predicates.drain(..);
            while let Some(predicate) = predicates.next() {
                let pv = predicate_at(&predicate);
                match predicate.op {
                    Op::Ex => {
                        if ((start.matches_exact_inner(pv) && start.op != Op::Gt)
                            || (start.matches_greater_inner(pv) && start.op != Op::Ex))
                            && !end.matches_greater_inner(pv)
                        {
                            start = predicate.clone();
                            end = predicate;
                        } else {
                            // Nothing will ever match this combination.
                            drop(predicates);
                            drop(range);
                            self.ranges.swap_remove(i);
                            has_empty = true;
                            continue 'or;
                        }
                    }
                    Op::Gt | Op::GtEq => {
                        // Choose the stricter requirement:
                        if start.matches_greater_inner(pv) {
                            // We have a higher starting version, so we're stricter.
                            start = predicate;
                        } else {
                            // The existing starting version is stricter.
                            // But, if we can "upgrade" it from GtEq to Gt, do so:
                            if start.op == Op::GtEq
                                && predicate.op == Op::Gt
                                && start.matches_exact_inner(pv)
                            {
                                start.op = Op::Gt;
                            }
                        }
                    }
                    Op::Lt | Op::LtEq => {
                        // Choose the stricter requirement:
                        if predicate.matches_greater_inner(predicate_at(&end)) {
                            // The current end includes us, so we're stricter.
                            end = predicate;
                        } else {
                            // The existing end point is probably stricter.
                            // Except if it is a <= and we're a <
                            if end.op == Op::LtEq
                                && predicate.op == Op::Lt
                                && end.matches_exact_inner(pv)
                            {
                                end.op = Op::Lt;
                            }
                        }
                    }
                }

                if end.matches_greater_inner(predicate_at(&start)) {
                    // This predicate will never match.
                    drop(predicates);
                    drop(range);
                    self.ranges.swap_remove(i);
                    has_empty = true;
                    continue 'or;
                }
            }
            drop(predicates);

            // If we get here, we have found the smallest range that this predicate covers, and it
            // is non-empty.
            range.predicates.push(start);
            range.predicates.push(end);
        }

        if has_empty && self.ranges.is_empty() {
            // This requirement will never match.
            self.ranges.push(Range {
                predicates: Vec::new(),
                compat: self.compat,
            });
            return;
        }

        // Now we have an OR of single predicates, and we want to make sure we only have one range
        // for each disjoint part of the version space. For example, if we have:
        //
        //   [ [>=1,<2], [>=1.5,<2] ]
        //
        // that can be simplified to just
        //
        //   [ [>=1.5,<2] ]
        //
        // Essentially we want to merge overlapping ranges.
        // We do this by sorting the predicates by their start version, and then merging adjacent
        // ranges.
        self.ranges
            .sort_by(|a, b| predicate_at(&a.predicates[0]).cmp(&predicate_at(&b.predicates[0])));

        // NOTE: we start from 2, since we're going to be looking at (n-i) and (n-i+1)
        for i in 2..=self.ranges.len() {
            // We need to walk backwards since we might merge some.
            let i = n - i;
            let ranges = &mut self.ranges[i..=(i + 1)];

            // The two ranges ([0] and [1]) overlap (and thus can be merged) if the start of [1]
            // falls before the end of [0].
            //
            // Recall that we push(start) and *then* push(end) in the simplification above, so
            // [i].predicates[0] is the start of each range.
            let i_end = &ranges[0].predicates[1];
            let j_start = &ranges[1].predicates[0];
            if j_start.matches_greater_inner(predicate_at(&i_end)) {
                // [0]'s end is strictly greater than [1]'s start, so the ranges overlap.
                drop(i_end);
                drop(j_start);
                drop(ranges);
                let mut j = self.ranges.swap_remove(i + 1);
                // Recall that .predicates[1] is the range end.
                let i_end = &mut self.ranges[i].predicates[1];
                let j_end = j.predicates.swap_remove(1);
                // We now use whichever end is greater of the two ranges.
                // Remember that while j starts after i, i might still have a later end!
                if j_end.matches_greater_inner(predicate_at(&i_end)) {
                    // i's end is strictly greater than j's end, so we prefer i's current end
                    drop(j_end);
                } else if j_end.matches_exact_inner(predicate_at(&i_end)) {
                    // i and j both end at the same version.
                    // Prefer the one with a stricter op
                    if j_end.op == Op::Ex || (j_end.op == Op::Lt && i_end.op != Op::Ex) {
                        // j's end is stricter.
                        *i_end = j_end;
                    } else {
                        // i's end is stricter, or they are equal so we can keep i
                        drop(j_end);
                    }
                } else {
                    // j's end is strictly greater than i's end, so we prefer j's end
                    // and thus expand the range.
                    *i_end = j_end;
                }
            } else if (i_end.op == Op::LtEq || i_end.op == Op::Ex)
                && (j_start.op == Op::GtEq || j_start.op == Op::Ex)
                && j_start.matches_exact_inner(predicate_at(&i_end))
            {
                // The two ranges are perfectly adjacent, so we can merge them
                drop(i_end);
                drop(j_start);
                drop(ranges);
                let mut j = self.ranges.swap_remove(i + 1);
                // Recall that .predicates[1] is the range end.
                let i_end = &mut self.ranges[i].predicates[1];
                let j_end = j.predicates.swap_remove(1);
                *i_end = j_end;
            } else {
                // The two ranges are disjoint and cannot be merged.
            }
        }

        // And finally, we do a pass to eliminate unnecessary start/end bounds that we injected to
        // make our lives easier above.
        //
        // NOTE: It _could_ be that [0] == bottom and [1] == top for a given range.
        // If that happens, we have a `*` requirement, which is expressed as >=0.0.0.
        // Should that be the case, we can really eliminate all the other ORs.
        let mut has_match_all = false;
        let unbounded_start = Predicate {
            op: Op::GtEq,
            major: 0,
            minor: 0,
            patch: 0,
            pre: Vec::new(),
        };
        for range in &mut self.ranges {
            if range.predicates[0] == range.predicates[1] {
                range.predicates.truncate(1);
                continue;
            }

            if range.predicates.last().unwrap().major == std::u64::MAX {
                let _ = range.predicates.pop();

                if range.predicates[0] == unbounded_start {
                    has_match_all = true;
                }
            } else if range.predicates[0] == unbounded_start {
                range.predicates.swap_remove(0);
            }
        }

        if has_match_all {
            self.ranges.truncate(1);
            self.ranges[0].predicates.clear();
            self.ranges[0].predicates.push(unbounded_start);
        }
    }
}

fn predicate_at<'a>(x: &'a Predicate) -> (u64, u64, u64, &'a [Identifier]) {
    (x.major, x.minor, x.patch, &x.pre)
}

impl str::FromStr for VersionReq {
    type Err = ReqParseError;

    fn from_str(s: &str) -> Result<VersionReq, ReqParseError> {
        VersionReq::parse(s)
    }
}

impl Range {
    fn matches(&self, ver: &Version) -> bool {
        self.predicates.iter().all(|p| p.matches(ver))
    }

    fn pre_tag_is_compatible(&self, ver: &Version) -> bool {
        self.predicates.iter().any(|p| p.pre_tag_is_compatible(ver))
    }
}

impl Predicate {
    fn exact(version: &Version) -> Predicate {
        Predicate {
            op: Ex,
            major: version.major,
            minor: version.minor,
            patch: version.patch,
            pre: version.pre.clone(),
        }
    }

    /// `matches()` takes a `Version` and determines if it matches this particular `Predicate`.
    pub fn matches(&self, ver: &Version) -> bool {
        match self.op {
            Ex => self.matches_exact(ver),
            Gt => self.matches_greater(ver),
            GtEq => self.matches_exact(ver) || self.matches_greater(ver),
            Lt => !self.matches_exact(ver) && !self.matches_greater(ver),
            LtEq => !self.matches_greater(ver),
        }
    }

    fn matches_exact_inner(
        &self,
        (major, minor, patch, pre): (u64, u64, u64, &[Identifier]),
    ) -> bool {
        self.major == major && self.minor == minor && self.patch == patch && &*self.pre == pre
    }

    fn matches_exact(&self, ver: &Version) -> bool {
        self.matches_exact_inner((ver.major, ver.minor, ver.patch, &ver.pre))
    }

    // https://docs.npmjs.com/misc/semver#prerelease-tags
    fn pre_tag_is_compatible(&self, ver: &Version) -> bool {
        // If a version has a prerelease tag (for example, 1.2.3-alpha.3) then it will
        // only be
        // allowed to satisfy comparator sets if at least one comparator with the same
        // [major,
        // minor, patch] tuple also has a prerelease tag.
        !ver.is_prerelease()
            || (self.major == ver.major
                && self.minor == ver.minor
                && self.patch == ver.patch
                && !self.pre.is_empty())
    }

    // Returns true if the passed-in version is strictly greater than self.
    fn matches_greater_inner(
        &self,
        (major, minor, patch, pre): (u64, u64, u64, &[Identifier]),
    ) -> bool {
        if self.major != major {
            return major > self.major;
        }

        if self.minor != minor {
            return minor > self.minor;
        }

        if self.patch != patch {
            return patch > self.patch;
        }

        if !self.pre.is_empty() {
            return pre.is_empty() || pre > &self.pre;
        }

        false
    }

    fn matches_greater(&self, ver: &Version) -> bool {
        self.matches_greater_inner((ver.major, ver.minor, ver.patch, &ver.pre))
    }

    fn has_exactly_one_match(&self) -> bool {
        self.op == Ex
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if self.ranges.is_empty() {
            write!(fmt, "*")?;
        } else {
            for (i, ref pred) in self.ranges.iter().enumerate() {
                if i == 0 {
                    write!(fmt, "{}", pred)?;
                } else {
                    write!(fmt, " || {}", pred)?;
                }
            }
        }

        Ok(())
    }
}

impl fmt::Display for Range {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, ref pred) in self.predicates.iter().enumerate() {
            if i == 0 {
                write!(fmt, "{}", pred)?;
            } else if self.compat == Compat::Npm {
                // Node does not expect commas between predicates
                write!(fmt, " {}", pred)?;
            } else {
                write!(fmt, ", {}", pred)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{}{}.{}.{}",
            self.op, self.major, self.minor, self.patch
        )?;

        if !self.pre.is_empty() {
            write!(fmt, "-")?;
            for (i, x) in self.pre.iter().enumerate() {
                if i != 0 {
                    write!(fmt, ".")?
                }
                write!(fmt, "{}", x)?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ex => write!(fmt, "=")?,
            Gt => write!(fmt, ">")?,
            GtEq => write!(fmt, ">=")?,
            Lt => write!(fmt, "<")?,
            LtEq => write!(fmt, "<=")?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::super::version::Version;
    use super::{Compat, Op, VersionReq};
    use std::hash::{Hash, Hasher};

    fn req(s: &str) -> VersionReq {
        VersionReq::parse(s).unwrap()
    }

    fn req_npm(s: &str) -> VersionReq {
        VersionReq::parse_compat(s, Compat::Npm).unwrap()
    }

    fn version(s: &str) -> Version {
        match Version::parse(s) {
            Ok(v) => v,
            Err(e) => panic!("`{}` is not a valid version. Reason: {:?}", s, e),
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

    fn calculate_hash<T: Hash>(t: T) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    #[test]
    fn test_parsing_default() {
        let r = req("1.0.0");

        assert_eq!(r.to_string(), ">=1.0.0, <2.0.0".to_string());

        assert_match(&r, &["1.0.0", "1.0.1"]);
        assert_not_match(&r, &["0.9.9", "0.10.0", "0.1.0"]);
    }

    #[test]
    fn test_parsing_default_npm() {
        let r = req_npm("1.0.0");

        assert_eq!(r.to_string(), "=1.0.0".to_string());

        assert_match(&r, &["1.0.0"]);
        assert_not_match(&r, &["0.9.9", "0.10.0", "0.1.0", "1.0.1"]);
    }

    #[test]
    fn test_parsing_exact() {
        let r = req("=1.0.0");

        assert!(r.to_string() == "=1.0.0".to_string());
        assert_eq!(r.to_string(), "=1.0.0".to_string());

        assert_match(&r, &["1.0.0"]);
        assert_not_match(&r, &["1.0.1", "0.9.9", "0.10.0", "0.1.0", "1.0.0-pre"]);

        let r = req("=0.9.0");

        assert_eq!(r.to_string(), "=0.9.0".to_string());

        assert_match(&r, &["0.9.0"]);
        assert_not_match(&r, &["0.9.1", "1.9.0", "0.0.9"]);

        let r = req("=0.1.0-beta2.a");

        assert_eq!(r.to_string(), "=0.1.0-beta2.a".to_string());

        assert_match(&r, &["0.1.0-beta2.a"]);
        assert_not_match(&r, &["0.9.1", "0.1.0", "0.1.1-beta2.a", "0.1.0-beta2"]);
    }

    #[test]
    fn test_parse_metadata_see_issue_88_see_issue_88() {
        for op in &[Op::Ex, Op::Gt, Op::GtEq, Op::Lt, Op::LtEq] {
            println!("{} 1.2.3+meta", op);
            req(&format!("{} 1.2.3+meta", op));
        }
    }

    #[test]
    pub fn test_parsing_greater_than() {
        let r = req(">= 1.0.0");

        assert_eq!(r.to_string(), ">=1.0.0".to_string());

        assert_match(&r, &["1.0.0", "2.0.0"]);
        assert_not_match(&r, &["0.1.0", "0.0.1", "1.0.0-pre", "2.0.0-pre"]);

        // https://github.com/steveklabnik/semver/issues/53
        let r = req(">= 2.1.0-alpha2");

        assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha3", "2.1.0", "3.0.0"]);
        assert_not_match(
            &r,
            &["2.0.0", "2.1.0-alpha1", "2.0.0-alpha2", "3.0.0-alpha2"],
        );
    }

    #[test]
    pub fn test_parsing_less_than() {
        let r = req("< 1.0.0");

        assert_eq!(r.to_string(), "<1.0.0".to_string());

        assert_match(&r, &["0.1.0", "0.0.1"]);
        assert_not_match(&r, &["1.0.0", "1.0.0-beta", "1.0.1", "0.9.9-alpha"]);

        let r = req("<= 2.1.0-alpha2");

        assert_match(&r, &["2.1.0-alpha2", "2.1.0-alpha1", "2.0.0", "1.0.0"]);
        assert_not_match(
            &r,
            &["2.1.0", "2.2.0-alpha1", "2.0.0-alpha2", "1.0.0-alpha2"],
        );
    }

    #[test]
    pub fn test_multiple() {
        let r = req("> 0.0.9, <= 2.5.3");
        assert_eq!(r.to_string(), ">0.0.9, <=2.5.3".to_string());
        assert_match(&r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_not_match(&r, &["0.0.8", "2.5.4"]);

        let r = req("0.3.0, 0.4.0");
        assert_eq!(
            r.to_string(),
            ">=0.3.0, <0.4.0, >=0.4.0, <0.5.0".to_string()
        );
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.4.0"]);

        let r = req("<= 0.2.0, >= 0.5.0");
        assert_eq!(r.to_string(), "<=0.2.0, >=0.5.0".to_string());
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.5.1"]);

        let r = req("0.1.0, 0.1.4, 0.1.6");
        assert_eq!(
            r.to_string(),
            ">=0.1.0, <0.2.0, >=0.1.4, <0.2.0, >=0.1.6, <0.2.0".to_string()
        );
        assert_match(&r, &["0.1.6", "0.1.9"]);
        assert_not_match(&r, &["0.1.0", "0.1.4", "0.2.0"]);

        assert!(VersionReq::parse("> 0.1.0,").is_err());
        assert!(VersionReq::parse("> 0.3.0, ,").is_err());

        let r = req(">=0.5.1-alpha3, <0.6");
        assert_eq!(r.to_string(), ">=0.5.1-alpha3, <0.6.0".to_string());
        assert_match(
            &r,
            &[
                "0.5.1-alpha3",
                "0.5.1-alpha4",
                "0.5.1-beta",
                "0.5.1",
                "0.5.5",
            ],
        );
        assert_not_match(
            &r,
            &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre"],
        );
        assert_not_match(&r, &["0.6.0", "0.6.0-pre"]);

        // https://github.com/steveklabnik/semver/issues/56
        let r = req("1.2.3 - 2.3.4");
        assert_eq!(r.to_string(), ">=1.2.3, <=2.3.4");
        assert_match(&r, &["1.2.3", "1.2.10", "2.0.0", "2.3.4"]);
        assert_not_match(&r, &["1.0.0", "1.2.2", "1.2.3-alpha1", "2.3.5"]);
    }

    // https://github.com/steveklabnik/semver/issues/55
    #[test]
    pub fn test_whitespace_delimited_comparator_sets() {
        let r = req("> 0.0.9 <= 2.5.3");
        assert_eq!(r.to_string(), ">0.0.9, <=2.5.3".to_string());
        assert_match(&r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_not_match(&r, &["0.0.8", "2.5.4"]);
    }

    #[test]
    pub fn test_multiple_npm() {
        let r = req_npm("> 0.0.9, <= 2.5.3");
        assert_eq!(r.to_string(), ">0.0.9 <=2.5.3".to_string());
        assert_match(&r, &["0.0.10", "1.0.0", "2.5.3"]);
        assert_not_match(&r, &["0.0.8", "2.5.4"]);

        let r = req_npm("0.3.0, 0.4.0");
        assert_eq!(r.to_string(), "=0.3.0 =0.4.0".to_string());
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.4.0"]);

        let r = req_npm("<= 0.2.0, >= 0.5.0");
        assert_eq!(r.to_string(), "<=0.2.0 >=0.5.0".to_string());
        assert_not_match(&r, &["0.0.8", "0.3.0", "0.5.1"]);

        let r = req_npm("0.1.0, 0.1.4, 0.1.6");
        assert_eq!(r.to_string(), "=0.1.0 =0.1.4 =0.1.6".to_string());
        assert_not_match(&r, &["0.1.0", "0.1.4", "0.1.6", "0.2.0"]);

        assert!(VersionReq::parse("> 0.1.0,").is_err());
        assert!(VersionReq::parse("> 0.3.0, ,").is_err());

        let r = req_npm(">=0.5.1-alpha3, <0.6");
        assert_eq!(r.to_string(), ">=0.5.1-alpha3 <0.6.0".to_string());
        assert_match(
            &r,
            &[
                "0.5.1-alpha3",
                "0.5.1-alpha4",
                "0.5.1-beta",
                "0.5.1",
                "0.5.5",
            ],
        );
        assert_not_match(
            &r,
            &["0.5.1-alpha1", "0.5.2-alpha3", "0.5.5-pre", "0.5.0-pre"],
        );
        assert_not_match(&r, &["0.6.0", "0.6.0-pre"]);
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
        assert_match(
            &r,
            &[
                "0.5.1-alpha3",
                "0.5.1-alpha4",
                "0.5.1-beta",
                "0.5.1",
                "0.5.5",
            ],
        );
        assert_not_match(
            &r,
            &[
                "0.5.1-alpha1",
                "0.5.2-alpha3",
                "0.5.5-pre",
                "0.5.0-pre",
                "0.6.0",
            ],
        );

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
        assert_match(
            &r,
            &["1.4.2", "1.4.3", "1.4.2-beta.5", "1.4.2-beta.6", "1.4.2-c"],
        );
        assert_not_match(
            &r,
            &[
                "0.9.9",
                "2.0.0",
                "1.4.2-alpha",
                "1.4.2-beta.4",
                "1.4.3-beta.5",
            ],
        );
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

    // https://github.com/steveklabnik/semver/issues/57
    #[test]
    pub fn test_parsing_logical_or() {
        let r = req("=1.2.3 || =2.3.4");
        assert_eq!(r.to_string(), "=1.2.3 || =2.3.4".to_string());
        assert_match(&r, &["1.2.3", "2.3.4"]);
        assert_not_match(&r, &["1.0.0", "2.9.0", "0.1.4"]);
        assert_not_match(&r, &["1.2.3-beta1", "2.3.4-alpha", "1.2.3-pre"]);

        let r = req("1.1 || =1.2.3");
        assert_eq!(r.to_string(), ">=1.1.0, <1.2.0 || =1.2.3".to_string());
        assert_match(&r, &["1.1.0", "1.1.12", "1.2.3"]);
        assert_not_match(&r, &["1.0.0", "1.2.2", "1.3.0"]);

        let r = req("6.* || 8.* || >= 10.*");
        assert_eq!(
            r.to_string(),
            ">=6.0.0, <7.0.0 || >=8.0.0, <9.0.0 || >=10.0.0".to_string()
        );
        assert_match(&r, &["6.0.0", "6.1.2"]);
        assert_match(&r, &["8.0.0", "8.2.4"]);
        assert_match(&r, &["10.1.2", "11.3.4"]);
        assert_not_match(&r, &["5.0.0", "7.0.0", "9.0.0"]);
    }

    #[test]
    pub fn test_parsing_logical_or_npm() {
        let r = req_npm("=1.2.3 || =2.3.4");
        assert_eq!(r.to_string(), "=1.2.3 || =2.3.4".to_string());
        assert_match(&r, &["1.2.3", "2.3.4"]);
        assert_not_match(&r, &["1.0.0", "2.9.0", "0.1.4"]);
        assert_not_match(&r, &["1.2.3-beta1", "2.3.4-alpha", "1.2.3-pre"]);

        let r = req_npm("1.1 || =1.2.3");
        assert_eq!(r.to_string(), ">=1.1.0 <1.2.0 || =1.2.3".to_string());
        assert_match(&r, &["1.1.0", "1.1.12", "1.2.3"]);
        assert_not_match(&r, &["1.0.0", "1.2.2", "1.3.0"]);

        let r = req_npm("6.* || 8.* || >= 10.*");
        assert_eq!(
            r.to_string(),
            ">=6.0.0 <7.0.0 || >=8.0.0 <9.0.0 || >=10.0.0".to_string()
        );
        assert_match(&r, &["6.0.0", "6.1.2"]);
        assert_match(&r, &["8.0.0", "8.2.4"]);
        assert_match(&r, &["10.1.2", "11.3.4"]);
        assert_not_match(&r, &["5.0.0", "7.0.0", "9.0.0"]);
    }

    #[test]
    pub fn test_any() {
        let r = VersionReq::any();
        assert_match(&r, &["0.0.1", "0.1.0", "1.0.0"]);
    }

    #[test]
    pub fn test_pre() {
        let r = req("=2.1.1-really.0");
        assert_match(&r, &["2.1.1-really.0"]);
    }

    // #[test]
    // pub fn test_parse_errors() {
    //    assert_eq!(Err(InvalidVersionRequirement), VersionReq::parse("\0"));
    //    assert_eq!(Err(OpAlreadySet), VersionReq::parse(">= >= 0.0.2"));
    //    assert_eq!(Err(InvalidSigil), VersionReq::parse(">== 0.0.2"));
    //    assert_eq!(Err(VersionComponentsMustBeNumeric),
    //               VersionReq::parse("a.0.0"));
    //    assert_eq!(Err(InvalidIdentifier), VersionReq::parse("1.0.0-"));
    //    assert_eq!(Err(MajorVersionRequired), VersionReq::parse(">="));
    // }

    #[test]
    pub fn test_from_str() {
        assert_eq!(
            "1.0.0".parse::<VersionReq>().unwrap().to_string(),
            ">=1.0.0, <2.0.0".to_string()
        );
        assert_eq!(
            "=1.0.0".parse::<VersionReq>().unwrap().to_string(),
            "=1.0.0".to_string()
        );
        assert_eq!(
            "~1".parse::<VersionReq>().unwrap().to_string(),
            ">=1.0.0, <2.0.0".to_string()
        );
        assert_eq!(
            "~1.2".parse::<VersionReq>().unwrap().to_string(),
            ">=1.2.0, <1.3.0".to_string()
        );
        assert_eq!(
            "^1".parse::<VersionReq>().unwrap().to_string(),
            ">=1.0.0, <2.0.0".to_string()
        );
        assert_eq!(
            "^1.1".parse::<VersionReq>().unwrap().to_string(),
            ">=1.1.0, <2.0.0".to_string()
        );
        assert_eq!(
            "*".parse::<VersionReq>().unwrap().to_string(),
            ">=0.0.0".to_string()
        );
        assert_eq!(
            "1.*".parse::<VersionReq>().unwrap().to_string(),
            ">=1.0.0, <2.0.0".to_string()
        );
        assert_eq!(
            "< 1.0.0".parse::<VersionReq>().unwrap().to_string(),
            "<1.0.0".to_string()
        );
    }

    // #[test]
    // pub fn test_from_str_errors() {
    //    assert_eq!(Err(InvalidVersionRequirement), "\0".parse::<VersionReq>());
    //    assert_eq!(Err(OpAlreadySet), ">= >= 0.0.2".parse::<VersionReq>());
    //    assert_eq!(Err(InvalidSigil), ">== 0.0.2".parse::<VersionReq>());
    //    assert_eq!(Err(VersionComponentsMustBeNumeric),
    //               "a.0.0".parse::<VersionReq>());
    //    assert_eq!(Err(InvalidIdentifier), "1.0.0-".parse::<VersionReq>());
    //    assert_eq!(Err(MajorVersionRequired), ">=".parse::<VersionReq>());
    // }

    #[test]
    fn test_cargo3202() {
        let v = "0.*.*".parse::<VersionReq>().unwrap();
        assert_eq!(">=0.0.0, <1.0.0", format!("{}", v.ranges[0]));

        let v = "0.0.*".parse::<VersionReq>().unwrap();
        assert_eq!(">=0.0.0, <0.1.0", format!("{}", v.ranges[0]));

        let r = req("0.*.*");
        assert_match(&r, &["0.5.0"]);
    }

    #[test]
    fn test_eq_hash() {
        assert!(req("^1") == req("^1"));
        assert!(calculate_hash(req("^1")) == calculate_hash(req("^1")));
        assert!(req("^1") != req("^2"));
    }

    #[test]
    fn test_ordering() {
        assert!(req("=1") > req("*"));
        assert!(req(">1") < req("*"));
        assert!(req(">=1") > req("*"));
        assert!(req("<1") > req("*"));
        assert!(req("<=1") > req("*"));
        assert!(req("~1") > req("*"));
        assert!(req("^1") > req("*"));
        assert!(req("*") == req("*"));
    }

    #[test]
    fn is_exact() {
        assert!(req("=1.0.0").is_exact());
        assert!(req("=1.0.0-alpha").is_exact());

        assert!(!req("=1").is_exact());
        assert!(!req(">=1.0.0").is_exact());
        assert!(!req(">=1.0.0, <2.0.0").is_exact());
    }

    fn simplify(mut v: VersionReq) -> VersionReq {
        v.simplify();
        v
    }

    #[test]
    fn simplify_preserve() {
        assert_eq!(simplify(req("=1.0.0")), req("=1.0.0"));
        assert_eq!(simplify(req(">1")), req(">1"));
        assert_eq!(simplify(req(">=1")), req(">=1"));
        assert_eq!(simplify(req("<1")), req("<1"));
        assert_eq!(simplify(req("<=1")), req("<=1"));
        assert_eq!(simplify(req("~1")), req("~1"));
        assert_eq!(simplify(req("^1")), req("^1"));
        assert_eq!(simplify(req("*")), req("*"));

        assert_eq!(simplify(req(">=1.0.0, <2.0.0")), req(">=1.0.0, <2.0.0"));
        assert_eq!(simplify(req("<1.0.0 || >3.0.0")), req("<1.0.0 || >3.0.0"));
        assert_eq!(simplify(req("<1.0.0 || =3.0.0")), req("<1.0.0 || =3.0.0"));
    }

    #[test]
    fn simplify_overlaps() {
        // Simplify away the <2.0.0 that 1 generates.
        assert_eq!(simplify(req("1, <1.5.0")), req(">=1.0.0, <1.5.0"));
        // Simplify down to = if exists
        assert_eq!(simplify(req("=1.0.0, <1.5.0")), req("=1.0.0"));
        // Simplify overlapping ORed ranges by start
        assert_eq!(simplify(req(">1.0.0 || >1.5.0")), req(">1.0.0"));
        // Simplify overlapping ORed ranges by end
        assert_eq!(simplify(req("<1.0.0 || <1.5.0")), req("<1.5.0"));
        // Simplify overlapping ORed ranges
        assert_eq!(
            simplify(req(">=1.0.0, <1.2.0 || >=1.1.0, <1.5.0")),
            req(">=1.0.0, <1.5.0")
        );
        // Simplify barely overlapping ORed ranges
        assert_eq!(
            simplify(req(">=1.0.0, <=1.2.0 || >=1.2.0, <1.5.0")),
            req(">=1.0.0, <1.5.0")
        );
        // Simplify down to something that matches nothing if intersection is empty
        let r = simplify(req("=1.0.0, =1.5.0"));
        assert_not_match(&r, &["1.0.0", "1.5.1"]);
    }

    #[test]
    fn dont_oversimplify() {
        // Don't simplfy a single-element range to equality (remember pre- versions).
        assert_ne!(simplify(req(">=1.2.0, <=1.2.0")), req("=1.2.0"));
        // Don't simplify not-quite overlapping ORed ranges
        assert_eq!(
            simplify(req(">=1.0.0, <=1.2.0 || >1.2.0, <1.5.0")),
            req(">=1.0.0, <=1.2.0 || >1.2.0, <1.5.0"),
        );
    }
}
