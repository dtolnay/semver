#![doc(html_root_url = "https://docs.rs/semver/0.0.0")]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(all(not(feature = "std"), not(no_alloc_crate)), no_std)]
#![cfg_attr(not(no_unsafe_op_in_unsafe_fn_lint), deny(unsafe_op_in_unsafe_fn))]
#![cfg_attr(no_unsafe_op_in_unsafe_fn_lint, allow(unused_unsafe))]
#![cfg_attr(no_str_strip_prefix, allow(unstable_name_collisions))]
#![allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::doc_markdown,
    clippy::items_after_statements,
    clippy::match_bool,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::needless_doctest_main,
    clippy::option_if_let_else,
    clippy::ptr_as_ptr,
    clippy::redundant_else,
    clippy::similar_names,
    clippy::unnested_or_patterns,
    clippy::unseparated_literal_suffix,
    clippy::wildcard_imports
)]

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

/// **SemVer version** as defined by <https://semver.org>.
///
/// # Syntax
///
/// - The major, minor, and patch numbers may be any integer 0 through u64::MAX.
///   When representing a SemVer version as a string, each number is written as
///   a base 10 integer. For example, `1.0.119`.
///
/// - Leading zeros are forbidden in those positions. For example `1.01.00` is
///   invalid as a SemVer version.
///
/// - The pre-release identifier, if present, must conform to the syntax
///   documented for [`Prerelease`].
///
/// - The build metadata, if present, must conform to the syntax documneted for
///   [`BuildMetadata`].
///
/// - Whitespace is not allowed anywhere in the version.
///
/// # Total ordering
///
/// Given any two SemVer versions, one is less than, greater than, or equal to
/// the other. Versions may be compared against one another using Rust's usual
/// comparison operators.
///
/// - The major, minor, and patch number are compared numerically from left to
/// right, lexicographically ordered as a 3-tuple of integers. So for example
/// version `1.5.0` is less than version `1.19.0`, despite the fact that
/// "1.19.0" &lt; "1.5.0" as ASCIIbetically compared strings and 1.19 &lt; 1.5
/// as real numbers.
///
/// - When major, minor, and patch are equal, a pre-release version is
///   considered less than the ordinary release:&ensp;version `1.0.0-alpha.1` is
///   less than version `1.0.0`.
///
/// - Two pre-releases of the same major, minor, patch are compared by
///   lexicographic ordering of dot-separated components of the pre-release
///   string.
///
///   - Identifiers consisting of only digits are compared
///     numerically:&ensp;`1.0.0-pre.8` is less than `1.0.0-pre.12`.
///
///   - Identifiers that contain a letter or hyphen are compared in ASCII sort
///     order:&ensp;`1.0.0-pre12` is less than `1.0.0-pre8`.
///
///   - Any numeric identifier is always less than any non-numeric
///     identifier:&ensp;`1.0.0-pre.1` is less than `1.0.0-pre.x`.
///
/// Example:&ensp;`1.0.0-alpha`&ensp;&lt;&ensp;`1.0.0-alpha.1`&ensp;&lt;&ensp;`1.0.0-alpha.beta`&ensp;&lt;&ensp;`1.0.0-beta`&ensp;&lt;&ensp;`1.0.0-beta.2`&ensp;&lt;&ensp;`1.0.0-beta.11`&ensp;&lt;&ensp;`1.0.0-rc.1`&ensp;&lt;&ensp;`1.0.0`
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Prerelease,
    pub build: BuildMetadata,
}

/// **SemVer version requirement** describing the intersection of some version
/// comparators, such as `>=1.2.3, <1.8`.
///
/// # Syntax
///
/// - Either `*` (meaning "any"), or one or more comma-separated comparators.
///
/// - A [`Comparator`] is an operator ([`Op`]) and a partial version, separated
///   by optional whitespace. For example `>=1.0.0` or `>=1.0`.
///
/// - Build metadata is syntactically permitted on the partial versions, but is
///   completely ignored, as it's never relevant to whether any comparator
///   matches a particular version.
///
/// - Whitespace is permitted around commas and around operators. Whitespace is
///   not permitted within a partial version, i.e. anywhere between the major
///   version number and its minor, patch, pre-release, or build metadata.
#[derive(Default, Clone, Eq, PartialEq, Hash, Debug)]
pub struct VersionReq {
    pub comparators: Vec<Comparator>,
}

/// A pair of comparison operator and partial version, such as `>=1.2`. Forms
/// one piece of a VersionReq.
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
    /// Create `Version` with an empty pre-release and build metadata.
    ///
    /// Equivalent to:
    ///
    /// ```
    /// # use semver::{BuildMetadata, Prerelease, Version};
    /// #
    /// # const fn new(major: u64, minor: u64, patch: u64) -> Version {
    /// Version {
    ///     major,
    ///     minor,
    ///     patch,
    ///     pre: Prerelease::EMPTY,
    ///     build: BuildMetadata::EMPTY,
    /// }
    /// # }
    /// ```
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }

    /// Create `Version` by parsing from string representation.
    ///
    /// # Errors
    ///
    /// Possible reasons for the parse to fail include:
    ///
    /// - `1.0` &mdash; too few numeric components. A SemVer version must have
    ///   exactly three. If you are looking at something that has fewer than
    ///   three numbers in it, it's possible it is a `VersionReq` instead (with
    ///   an implicit default `^` comparison operator).
    ///
    /// - `1.0.01` &mdash; a numeric component has a leading zero.
    ///
    /// - `1.0.unknown` &mdash; unexpected character in one of the components.
    ///
    /// - `1.0.0-` or `1.0.0+` &mdash; the pre-release or build metadata are
    ///   indicated present but empty.
    ///
    /// - `1.0.0-alpha_123` &mdash; pre-release or build metadata have something
    ///   outside the allowed characters, which are `0-9`, `A-Z`, `a-z`, `-`,
    ///   and `.` (dot).
    ///
    /// - `23456789999999999999.0.0` &mdash; overflow of a u64.
    pub fn parse(text: &str) -> Result<Self, Error> {
        Version::from_str(text)
    }
}

impl VersionReq {
    /// A `VersionReq` with no constraint on the version numbers it matches.
    /// Equivalent to `VersionReq::parse("*").unwrap()`.
    ///
    /// In terms of comparators this is equivalent to `>=0.0.0`.
    ///
    /// Counterintuitively a `*` VersionReq does not match every possible
    /// version number. In particular, in order for *any* `VersionReq` to match
    /// a pre-release version, the `VersionReq` must contain at least one
    /// `Comparator` that has an explicit major, minor, and patch version
    /// identical to the pre-release being matched, and that has a nonempty
    /// pre-release component. Since `*` is not written with an explicit major,
    /// minor, and patch version, and does not contain a nonempty pre-release
    /// component, it does not match any pre-release versions.
    #[cfg(not(no_const_vec_new))] // rustc <1.39
    pub const STAR: Self = VersionReq {
        comparators: Vec::new(),
    };

    /// Create `VersionReq` by parsing from string representation.
    ///
    /// # Errors
    ///
    /// Possible reasons for the parse to fail include:
    ///
    /// - `>a.b` &mdash; unexpected characters in the partial version.
    ///
    /// - `@1.0.0` &mdash; unrecognized comparison operator.
    ///
    /// - `^1.0.0, ` &mdash; unexpected end of input.
    ///
    /// - `>=1.0 <2.0` &mdash; missing comma between comparators.
    ///
    /// - `*.*` &mdash; unsupported wildcard syntax.
    pub fn parse(text: &str) -> Result<Self, Error> {
        VersionReq::from_str(text)
    }

    /// Evaluate whether the given `Version` satisfies the version requirement
    /// described by `self`.
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
