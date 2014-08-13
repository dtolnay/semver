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

#![crate_name = "semver"]
#![experimental]
#![crate_type = "rlib"]
#![crate_type = "dylib"]
#![license = "MIT/ASL2"]
#![doc(html_logo_url = "http://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
       html_favicon_url = "http://www.rust-lang.org/favicon.ico")]
#![feature(default_type_params)]
#![feature(macro_rules)]

use std::char;
use std::cmp;
use std::fmt::Show;
use std::fmt;
use std::hash;
use std::str::CharOffsets;

/// An identifier in the pre-release or build metadata. If the identifier can
/// be parsed as a decimal value, it will be represented with `Numeric`.
#[deriving(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(missing_doc)]
pub enum Identifier {
    Numeric(u64),
    AlphaNumeric(String)
}

impl fmt::Show for Identifier {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Numeric(ref n) => n.fmt(f),
            AlphaNumeric(ref s) => s.fmt(f)
        }
    }
}


/// Represents a version number conforming to the semantic versioning scheme.
#[deriving(Clone, Eq)]
pub struct Version {
    /// The major version, to be incremented on incompatible changes.
    pub major: u32,
    /// The minor version, to be incremented when functionality is added in a
    /// backwards-compatible manner.
    pub minor: u32,
    /// The patch version, to be incremented when backwards-compatible bug
    /// fixes are made.
    pub patch: u32,
    /// The pre-release version identifier, if one exists.
    pub pre: Vec<Identifier>,
    /// The build metadata, ignored when determining version precedence.
    pub build: Vec<Identifier>,
}

impl fmt::Show for Version {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "{}.{}.{}", self.major, self.minor, self.patch))
        if !self.pre.is_empty() {
            try!(write!(f, "-"));
            for (i, x) in self.pre.iter().enumerate() {
                if i != 0 { try!(write!(f, ".")) };
                try!(x.fmt(f));
            }
        }
        if !self.build.is_empty() {
            try!(write!(f, "+"));
            for (i, x) in self.build.iter().enumerate() {
                if i != 0 { try!(write!(f, ".")) };
                try!(x.fmt(f));
            }
        }
        Ok(())
    }
}

impl cmp::PartialEq for Version {
    #[inline]
    fn eq(&self, other: &Version) -> bool {
        // We should ignore build metadata here, otherwise versions v1 and v2
        // can exist such that !(v1 < v2) && !(v1 > v2) && v1 != v2, which
        // violate strict total ordering rules.
        self.major == other.major &&
            self.minor == other.minor &&
            self.patch == other.patch &&
            self.pre == other.pre
    }
}

impl cmp::PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match self.major.cmp(&other.major) {
            Equal => {}
            r => return r,
        }

        match self.minor.cmp(&other.minor) {
            Equal => {}
            r => return r,
        }

        match self.patch.cmp(&other.patch) {
            Equal => {}
            r => return r,
        }

        // NB: semver spec says 0.0.0-pre < 0.0.0
        // but the version of ord defined for vec
        // says that [] < [pre] so we alter it here
        match (self.pre.len(), other.pre.len()) {
            (0, 0) => Equal,
            (0, _) => Greater,
            (_, 0) => Less,
            (_, _) => self.pre.cmp(&other.pre)
        }
    }
}

impl<S: hash::Writer> hash::Hash<S> for Version {
    fn hash(&self, into: &mut S) {
        self.major.hash(into);
        self.minor.hash(into);
        self.patch.hash(into);
        self.pre.hash(into);
    }
}

fn take_nonempty_prefix<T:Iterator<char>>(rdr: &mut T, pred: |char| -> bool)
                        -> (String, Option<char>) {
    let mut buf = String::new();
    let mut ch = rdr.next();
    loop {
        match ch {
            None => break,
            Some(c) if !pred(c) => break,
            Some(c) => {
                buf.push_char(c);
                ch = rdr.next();
            }
        }
    }
    (buf, ch)
}

fn take_num<T: Iterator<char>>(rdr: &mut T) -> Option<(u32, Option<char>)> {
    let (s, ch) = take_nonempty_prefix(rdr, char::is_digit);
    match from_str::<u32>(s.as_slice()) {
        None => None,
        Some(i) => Some((i, ch))
    }
}

fn take_ident<T: Iterator<char>>(rdr: &mut T) -> Option<(Identifier, Option<char>)> {
    let (s,ch) = take_nonempty_prefix(rdr, char::is_alphanumeric);
    if s.as_slice().chars().all(char::is_digit) {
        match from_str::<u64>(s.as_slice()) {
            None => None,
            Some(i) => Some((Numeric(i), ch))
        }
    } else {
        Some((AlphaNumeric(s), ch))
    }
}

fn expect(ch: Option<char>, c: char) -> Option<()> {
    if ch != Some(c) {
        None
    } else {
        Some(())
    }
}

fn parse_iter<T: Iterator<char>>(rdr: &mut T) -> Option<Version> {
    let maybe_vers = take_num(rdr).and_then(|(major, ch)| {
        expect(ch, '.').and_then(|_| Some(major))
    }).and_then(|major| {
        take_num(rdr).and_then(|(minor, ch)| {
            expect(ch, '.').and_then(|_| Some((major, minor)))
        })
    }).and_then(|(major, minor)| {
        take_num(rdr).and_then(|(patch, ch)| {
           Some((major, minor, patch, ch))
        })
    });

    let (major, minor, patch, ch) = match maybe_vers {
        Some((a, b, c, d)) => (a, b, c, d),
        None => return None
    };

    let mut pre = vec!();
    let mut build = vec!();

    let mut ch = ch;
    if ch == Some('-') {
        loop {
            let (id, c) = match take_ident(rdr) {
                Some((id, c)) => (id, c),
                None => return None
            };
            pre.push(id);
            ch = c;
            if ch != Some('.') { break; }
        }
    }

    if ch == Some('+') {
        loop {
            let (id, c) = match take_ident(rdr) {
                Some((id, c)) => (id, c),
                None => return None
            };
            build.push(id);
            ch = c;
            if ch != Some('.') { break; }
        }
    }

    Some(Version {
        major: major,
        minor: minor,
        patch: patch,
        pre: pre,
        build: build,
    })
}

#[deriving(PartialEq,Show,PartialOrd)]
pub enum ParseError {
    NonAsciiIdentifier,
    IncorrectParse(Version, String),
    GenericFailure,
}

/// Parse a string into a semver object.
pub fn parse(s: &str) -> Result<Version, ParseError> {
    if !s.is_ascii() {
        return Err(NonAsciiIdentifier)
    }
    let s = s.trim();
    let v = parse_iter(&mut s.chars());
    match v {
        Some(v) => {
            if v.to_string().equiv(&s) {
                Ok(v)
            } else {
                Err(IncorrectParse(v, s.to_string()))
            }
        }
        None => Err(GenericFailure)
    }
}

#[deriving(PartialEq,Clone)]
pub struct VersionReq {
    predicates: Vec<Predicate>
}

#[deriving(PartialEq,Clone)]
enum Op {
    Ex,   // Exact
    Gt,   // Greater than
    GtEq, // Greater than or equal to
    Lt,   // Less than
    LtEq  // Less than or equal to
}

#[deriving(PartialEq,Clone)]
struct Predicate {
    op: Op,
    major: uint,
    minor: Option<uint>,
    patch: Option<uint>
}

struct PredBuilder {
    op: Option<Op>,
    major: Option<uint>,
    minor: Option<uint>,
    patch: Option<uint>
}


impl VersionReq {
    pub fn any() -> VersionReq {
        VersionReq { predicates: vec!() }
    }

    pub fn parse(input: &str) -> Result<VersionReq, String> {
        let mut lexer = Lexer::new(input);
        let mut builder = PredBuilder::new();
        let mut predicates = Vec::new();

        for token in lexer {
            match token {
                Sigil(x) => try!(builder.set_sigil(x)),
                AlphaNum(x) => try!(builder.set_version_part(x)),
                Dot => (), // Nothing to do for now
                _ => unimplemented!()
            }
        }

        if lexer.is_error() {
            return Err("invalid version requirement".to_string());
        }

        predicates.push(try!(builder.build()));

        Ok(VersionReq { predicates: predicates })
    }

    pub fn exact(version: &Version) -> VersionReq {
        VersionReq { predicates: vec!(Predicate::exact(version)) }
    }

    pub fn matches(&self, version: &Version) -> bool {
        self.predicates.iter().all(|p| p.matches(version))
    }
}

impl Predicate {
    pub fn exact(version: &Version) -> Predicate {
        Predicate {
            op: Ex,
            major: version.major,
            minor: Some(version.minor),
            patch: Some(version.patch)
        }
    }

    pub fn matches(&self, ver: &Version) -> bool {
        match self.op {
            Ex => self.is_exact(ver),
            Gt => self.is_greater(ver),
            GtEq => self.is_exact(ver) || self.is_greater(ver),
            _ => false // not implemented
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

        true
    }

    fn is_greater(self, ver: &Version) -> bool {
        if self.major != ver.major {
            return self.major > ver.major;
        }

        match self.minor {
            Some(minor) => {
                if minor != ver.minor {
                    return minor > ver.minor
                }
            }
            None => return false
        }

        match self.patch {
            Some(patch) => {
                if patch != ver.patch {
                    return patch > ver.patch
                }
            }

            None => return false
        }

        false
    }
}

impl PredBuilder {
    fn new() -> PredBuilder {
        PredBuilder {
            op: None,
            major: None,
            minor: None,
            patch: None
        }
    }

    fn set_sigil(&mut self, sigil: &str) -> Result<(), String> {
        if self.op.is_some() {
            return Err("op already set".to_string());
        }

        match Op::from_sigil(sigil) {
            Some(op) => self.op = Some(op),
            _ => return Err("invalid sigil".to_string())
        }

        Ok(())
    }

    fn set_version_part(&mut self, part: &str) -> Result<(), String> {
        if self.op.is_none() {
            // If no op is specified, then the predicate is an exact match on
            // the version
            self.op = Some(Ex);
        }

        if self.major.is_none() {
            self.major = Some(try!(parse_version_part(part)));
        }
        else if self.minor.is_none() {
            self.minor = Some(try!(parse_version_part(part)));
        }
        else if self.patch.is_none() {
            self.patch = Some(try!(parse_version_part(part)));
        }

        Ok(())
    }

    /// Validates that a version predicate can be created given the present
    /// information.
    fn build(&self) -> Result<Predicate, String> {
        let op = match self.op {
            Some(x) => x,
            None => return Err("op required".to_string())
        };

        let major = match self.major {
            Some(x) => x,
            None => return Err("major version required".to_string())
        };

        Ok(Predicate {
            op: op,
            major: major,
            minor: self.minor,
            patch: self.patch
        })
    }
}

struct Lexer<'a> {
    c: char,
    idx: uint,
    iter: CharOffsets<'a>,
    mark: Option<uint>,
    input: &'a str,
    state: LexState
}

#[deriving(Show,PartialEq)]
enum LexState {
    LexInit,
    LexStart,
    LexAlphaNum,
    LexSigil,
    LexErr,
    LexWin
}

#[deriving(Show)]
enum Token<'a> {
    Sigil(&'a str),
    AlphaNum(&'a str),
    Comma,
    Dot
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Lexer<'a> {
        Lexer {
            c: '\0',
            idx: 0,
            iter: input.char_indices(),
            mark: None,
            input: input,
            state: LexInit
        }
    }

    fn is_error(&self) -> bool {
        self.state == LexErr
    }

    fn mark(&mut self, at: uint) {
        self.mark = Some(at)
    }

    fn flush(&mut self, to: uint, kind: LexState) -> Option<Token<'a>> {
        match self.mark {
            Some(mark) => {
                if to <= mark {
                    return None;
                }

                let s = self.input.slice(mark, to);

                self.mark = None;

                match kind {
                    LexAlphaNum => Some(AlphaNum(s)),
                    LexSigil => Some(Sigil(s)),
                    _ => None // bug
                }
            }
            None => None
        }
    }
}

impl<'a> Iterator<Token<'a>> for Lexer<'a> {
    fn next(&mut self) -> Option<Token<'a>> {
        let mut c;
        let mut idx = 0;

        macro_rules! next(
            () => (
                match self.iter.next() {
                    Some((n_idx, n_char)) => {
                        c = n_char;
                        idx = n_idx;
                    }
                    _ => {
                      let s = self.state;
                      return self.flush(idx + 1, s)
                    }
                }
            ))

        macro_rules! flush(
            ($s:expr) => ({
                self.c = c;
                self.idx = idx;
                self.flush(idx, $s)
            }))


        if self.state == LexInit {
            self.state = LexStart;
            next!();
        } else {
            c = self.c;
            idx = self.idx;
        }

        loop {
            match self.state {
                LexStart => {
                    if c.is_whitespace() {
                        next!(); // Ignore
                    }
                    else if c.is_alphanumeric() {
                        self.mark(idx);
                        self.state = LexAlphaNum;
                        next!();
                    }
                    else if is_sigil(c) {
                        self.mark(idx);
                        self.state = LexSigil;
                        next!();
                    }
                    else if c == '.' {
                        self.state = LexInit;
                        return Some(Dot);
                    }
                    else if c == ',' {
                        self.state = LexInit;
                        return Some(Comma);
                    } else {
                        self.state = LexErr;
                        return None;
                    }
                }
                LexAlphaNum => {
                    if c.is_alphanumeric() {
                        next!();
                    } else {
                        self.state = LexStart;
                        return flush!(LexAlphaNum);
                    }
                }
                LexSigil => {
                    if is_sigil(c) {
                        next!();
                    } else {
                        self.state = LexStart;
                        return flush!(LexSigil);
                    }
                }
                LexErr | LexWin => return None,
                LexInit => return None // bug
            }
        }
    }
}

impl Op {
    fn from_sigil(sigil: &str) -> Option<Op> {
        match sigil {
            "=" => Some(Ex),
            ">" => Some(Gt),
            ">=" => Some(GtEq),
            "<" => Some(Lt),
            "<=" => Some(LtEq),
            _ => None
        }
    }
}

fn parse_version_part(s: &str) -> Result<uint, String> {
    let mut ret = 0;

    for c in s.chars() {
        let n = (c as uint) - ('0' as uint);

        if n > 9 {
            return Err("version components must be numeric".to_string());
        }

        ret *= 10;
        ret +=  n;
    }

    Ok(ret)
}

fn is_sigil(c: char) -> bool {
    match c {
        '>' | '<' | '=' | '~' | '^' => true,
        _ => false
    }
}

impl fmt::Show for VersionReq {
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

impl fmt::Show for Predicate {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{} {}", self.op, self.major));

        match self.minor {
            Some(v) => try!(write!(fmt, ".{}", v)),
            None => ()
        }

        match self.patch {
            Some(v) => try!(write!(fmt, ".{}", v)),
            None => ()
        }

        Ok(())
    }
}

impl fmt::Show for Op {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Ex => try!(write!(fmt, "=")),
            Gt => try!(write!(fmt, ">")),
            GtEq => try!(write!(fmt, ">=")),
            Lt => try!(write!(fmt, "<")),
            LtEq => try!(write!(fmt, "<="))
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::{
        VersionReq,
        parse,
        Version,
    };

    fn req(s: &str) -> VersionReq {
        VersionReq::parse(s).unwrap()
    }

    fn version(s: &str) -> Version {
        match parse(s) {
            Some(v) => v,
            None => fail!("`{}` is not a valid version", s)
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
    pub fn test_parsing_exact() {
        let r = req("1.0.0");

        assert!(r.to_string() == "= 1.0.0".to_string());

        assert_match(&r, ["1.0.0"]);
        assert_not_match(&r, ["1.0.1", "0.9.9", "0.10.0", "0.1.0"]);

        let r = req("0.9.0");

        assert!(r.to_string() == "= 0.9.0".to_string());

        assert_match(&r, ["0.9.0"]);
        assert_not_match(&r, ["0.9.1", "1.9.0", "0.0.9"]);
    }

    #[test]
    pub fn test_parsing_greater_than() {
        let r = req(">= 1.0.0");

        assert!(r.to_string() == ">= 1.0.0".to_string());

        assert_match(&r, ["1.0.0"]);
    }

    /* TODO:
     * - Test parse errors
     * - Handle pre releases
     */
}


#[test]
fn test_parse() {
    assert_eq!(parse(""), Err(GenericFailure));
    assert_eq!(parse("  "), Err(GenericFailure));
    assert_eq!(parse("1"),  Err(GenericFailure));
    assert_eq!(parse("1.2"), Err(GenericFailure));
    assert_eq!(parse("1.2"), Err(GenericFailure));
    assert_eq!(parse("1"), Err(GenericFailure));
    assert_eq!(parse("1.2"), Err(GenericFailure));
    assert_eq!(parse("1.2.3-"), Err(GenericFailure));
    assert_eq!(parse("a.b.c"), Err(GenericFailure));

    let version = Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(),
        build: vec!(),
    };
    let error = Err(IncorrectParse(version, "1.2.3 abc".to_string()));
    assert_eq!(parse("1.2.3 abc"), error);

    assert!(parse("1.2.3") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(),
        build: vec!(),
    }));
    assert!(parse("  1.2.3  ") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(),
        build: vec!(),
    }));
    assert!(parse("1.2.3-alpha1") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(AlphaNumeric("alpha1".to_string())),
        build: vec!(),
    }));
    assert!(parse("  1.2.3-alpha1  ") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(AlphaNumeric("alpha1".to_string())),
        build: vec!()
    }));
    assert!(parse("1.2.3+build5") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(),
        build: vec!(AlphaNumeric("build5".to_string()))
    }));
    assert!(parse("  1.2.3+build5  ") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(),
        build: vec!(AlphaNumeric("build5".to_string()))
    }));
    assert!(parse("1.2.3-alpha1+build5") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(AlphaNumeric("alpha1".to_string())),
        build: vec!(AlphaNumeric("build5".to_string()))
    }));
    assert!(parse("  1.2.3-alpha1+build5  ") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(AlphaNumeric("alpha1".to_string())),
        build: vec!(AlphaNumeric("build5".to_string()))
    }));
    assert!(parse("1.2.3-1.alpha1.9+build5.7.3aedf  ") == Ok(Version {
        major: 1u32,
        minor: 2u32,
        patch: 3u32,
        pre: vec!(Numeric(1),AlphaNumeric("alpha1".to_string()),Numeric(9)),
        build: vec!(AlphaNumeric("build5".to_string()),
                 Numeric(7),
                 AlphaNumeric("3aedf".to_string()))
    }));

}

#[test]
fn test_eq() {
    assert_eq!(parse("1.2.3"), parse("1.2.3"));
    assert_eq!(parse("1.2.3-alpha1"), parse("1.2.3-alpha1"));
    assert_eq!(parse("1.2.3+build.42"), parse("1.2.3+build.42"));
    assert_eq!(parse("1.2.3-alpha1+42"), parse("1.2.3-alpha1+42"));
    assert_eq!(parse("1.2.3+23"), parse("1.2.3+42"));
}

#[test]
fn test_ne() {
    assert!(parse("0.0.0")       != parse("0.0.1"));
    assert!(parse("0.0.0")       != parse("0.1.0"));
    assert!(parse("0.0.0")       != parse("1.0.0"));
    assert!(parse("1.2.3-alpha") != parse("1.2.3-beta"));
}

#[test]
fn test_show() {
    assert_eq!(format!("{}", parse("1.2.3").unwrap()),
               "1.2.3".to_string());
    assert_eq!(format!("{}", parse("1.2.3-alpha1").unwrap()),
               "1.2.3-alpha1".to_string());
    assert_eq!(format!("{}", parse("1.2.3+build.42").unwrap()),
               "1.2.3+build.42".to_string());
    assert_eq!(format!("{}", parse("1.2.3-alpha1+42").unwrap()),
               "1.2.3-alpha1+42".to_string());
}

#[test]
fn test_to_string() {
    assert_eq!(parse("1.2.3").unwrap().to_string(), "1.2.3".to_string());
    assert_eq!(parse("1.2.3-alpha1").unwrap().to_string(), "1.2.3-alpha1".to_string());
    assert_eq!(parse("1.2.3+build.42").unwrap().to_string(), "1.2.3+build.42".to_string());
    assert_eq!(parse("1.2.3-alpha1+42").unwrap().to_string(), "1.2.3-alpha1+42".to_string());
}

#[test]
fn test_lt() {
    assert!(parse("0.0.0")          < parse("1.2.3-alpha2"));
    assert!(parse("1.0.0")          < parse("1.2.3-alpha2"));
    assert!(parse("1.2.0")          < parse("1.2.3-alpha2"));
    assert!(parse("1.2.3-alpha1")   < parse("1.2.3"));
    assert!(parse("1.2.3-alpha1")   < parse("1.2.3-alpha2"));
    assert!(!(parse("1.2.3-alpha2") < parse("1.2.3-alpha2")));
    assert!(!(parse("1.2.3+23")     < parse("1.2.3+42")));
}

#[test]
fn test_le() {
    assert!(parse("0.0.0")        <= parse("1.2.3-alpha2"));
    assert!(parse("1.0.0")        <= parse("1.2.3-alpha2"));
    assert!(parse("1.2.0")        <= parse("1.2.3-alpha2"));
    assert!(parse("1.2.3-alpha1") <= parse("1.2.3-alpha2"));
    assert!(parse("1.2.3-alpha2") <= parse("1.2.3-alpha2"));
    assert!(parse("1.2.3+23")     <= parse("1.2.3+42"));
}

#[test]
fn test_gt() {
    assert!(parse("1.2.3-alpha2")   > parse("0.0.0"));
    assert!(parse("1.2.3-alpha2")   > parse("1.0.0"));
    assert!(parse("1.2.3-alpha2")   > parse("1.2.0"));
    assert!(parse("1.2.3-alpha2")   > parse("1.2.3-alpha1"));
    assert!(parse("1.2.3")          > parse("1.2.3-alpha2"));
    assert!(!(parse("1.2.3-alpha2") > parse("1.2.3-alpha2")));
    assert!(!(parse("1.2.3+23")     > parse("1.2.3+42")));
}

#[test]
fn test_ge() {
    assert!(parse("1.2.3-alpha2") >= parse("0.0.0"));
    assert!(parse("1.2.3-alpha2") >= parse("1.0.0"));
    assert!(parse("1.2.3-alpha2") >= parse("1.2.0"));
    assert!(parse("1.2.3-alpha2") >= parse("1.2.3-alpha1"));
    assert!(parse("1.2.3-alpha2") >= parse("1.2.3-alpha2"));
    assert!(parse("1.2.3+23")     >= parse("1.2.3+42"));
}

#[test]
fn test_spec_order() {
    let vs = ["1.0.0-alpha",
              "1.0.0-alpha.1",
              "1.0.0-alpha.beta",
              "1.0.0-beta",
              "1.0.0-beta.2",
              "1.0.0-beta.11",
              "1.0.0-rc.1",
              "1.0.0"];
    let mut i = 1;
    while i < vs.len() {
        let a = parse(vs[i-1]).unwrap();
        let b = parse(vs[i]).unwrap();
        assert!(a < b);
        i += 1;
    }
}
