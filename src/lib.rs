// Copyright 2012-2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Semantic version parsing and comparison.
//!
//! Semantic versioning (see http://semver.org/) is a set of rules for
//! assigning version numbers.
//!
//! ## SemVer overview
//!
//! Given a version number MAJOR.MINOR.PATCH, increment the:
//!
//! 1. MAJOR version when you make incompatible API changes,
//! 2. MINOR version when you add functionality in a backwards-compatible manner, and
//! 3. PATCH version when you make backwards-compatible bug fixes.
//!
//! Additional labels for pre-release and build metadata are available as extensions to the
//! MAJOR.MINOR.PATCH format.
//!
//! Any references to 'the spec' in this documentation refer to [version 2.0 of the SemVer
//! spec](http://semver.org/spec/v2.0.0.html).
//!
//! ## SemVer and the Rust ecosystem
//!
//! Rust itself follows the SemVer specification, as does its standard libraries. The two are
//! not tied together.
//!
//! [Cargo](http://crates.io), Rust's package manager, uses SemVer to determine which versions of
//! packages you need installed.
//!
//! ## Versions
//!
//! At its simplest, the `semver` crate allows you to construct `Version` objects using the `parse`
//! method:
//!
//! ```{rust}
//! use semver::Version;
//!
//! assert!(Version::parse("1.2.3") == Ok(Version {
//!    major: 1u32,
//!    minor: 2u32,
//!    patch: 3u32,
//!    pre: vec!(),
//!    build: vec!(),
//! }));
//! ```
//!
//! If you have multiple `Version`s, you can use the usual comparison operators to compare them:
//!
//! ```{rust}
//! use semver::Version;
//!
//! assert!(Version::parse("1.2.3-alpha")  != Version::parse("1.2.3-beta"));
//! assert!(Version::parse("1.2.3-alpha2") >  Version::parse("1.2.0"));
//! ```
//!
//! ## Ranges
//!
//! The `semver` crate also provides a `range` module, which allows you to do more
//! complex comparisons.
//!
//! For example, creating a requirement that only matches versions greater than or
//! equal to 1.0.0:
//!
//! ```{rust}
//! use semver::Version;
//! use semver::VersionReq;
//!
//! let r = VersionReq::parse(">= 1.0.0").unwrap();
//! let v = Version::parse("1.0.0").unwrap();
//!
//! assert!(r.to_string() == ">= 1.0.0".to_string());
//! assert!(r.matches(&v))
//! ```

#![crate_name = "semver"]
#![experimental]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico")]
#![feature(default_type_params)]
#![feature(macro_rules)]

// We take the common approach of keeping our own module system private, and
// just re-exporting the interface that we want.

pub use version::{
    Version,
    Identifier,
    ParseError,
};

pub use range::VersionReq;

// SemVer-compliant versions.
mod version;

// advanced version comparisons
mod range;

