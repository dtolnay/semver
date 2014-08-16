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
//! assigning version numbers intended to convey meaning about what has
//! changed, and how much. A version number has five parts:
//!
//!  * Major number, updated for incompatible API changes
//!  * Minor number, updated for backwards-compatible API additions
//!  * Patch number, updated for backwards-compatible bugfixes
//!  * Pre-release information (optional), preceded by a hyphen (`-`)
//!  * Build metadata (optional), preceded by a plus sign (`+`)
//!
//! The three mandatory components are required to be decimal numbers. The
//! pre-release information and build metadata are required to be a
//! period-separated list of identifiers containing only alphanumeric
//! characters and hyphens.
//!
//! An example version number with all five components is
//! `0.8.1-rc.3.0+20130922.linux`.
//!
//! There are two modules

#![crate_name = "semver"]
#![experimental]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico")]
#![feature(default_type_params)]
#![feature(macro_rules)]

pub mod version;
pub mod range;
