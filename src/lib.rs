#[macro_use]
extern crate nom;

pub mod parser;

use std::result;

/// A SemVer Version
#[derive(PartialEq,Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<String>,
    build: Option<String>,
}

/// An error type for this crate
///
/// Currently, just a generic error. Will make this nicer later.
#[derive(Debug)]
enum SemVerError {
    GenericError,
}

/// A Result type for errors
pub type Result<T> = result::Result<T, SemVerError>;

impl From<()> for SemVerError {
    fn from(_: ()) -> SemVerError {
        SemVerError::GenericError
    }
}

impl Version {
    /// Create a Version from a string
    ///
    /// Currently supported: x, x.y, and x.y.z versions.
    pub fn parse(version: &str) -> Result<Version> {
        Ok(try!(parser::try_parse(version.as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn parse_major_version() {
        let version = "10";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 10,
                       minor: 0,
                       patch: 0,
                       pre: None,
                       build: None,
                   });
    }

    #[test]
    fn parse_minor_version() {
        let version = "10.11";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 10,
                       minor: 11,
                       patch: 0,
                       pre: None,
                       build: None,
                   });
    }

    #[test]
    fn parse_version() {
        let version = "10.11.12";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 10,
                       minor: 11,
                       patch: 12,
                       pre: None,
                       build: None,
                   });
    }

    #[test]
    fn parse_pre_basic() {
        let version = "1.0.0-alpha";
        let version = Version::parse(version).unwrap();
        assert_eq!(version,
                   Version {
                       major: 1,
                       minor: 0,
                       patch: 0,
                       pre: Some(String::from("alpha")),
                       build: None,
                   });
    }

    #[test]
    fn parse_pre_dot() {
        let version = "1.0.0-alpha.1";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 1,
                       minor: 0,
                       patch: 0,
                       pre: Some(String::from("alpha.1")),
                       build: None,
                   });
    }

    #[test]
    fn parse_build() {
        let version = "1.0.0-alpha+100";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 1,
                       minor: 0,
                       patch: 0,
                       pre: Some(String::from("alpha")),
                       build: Some(String::from("100")),
                   });
    }

    #[test]
    fn parse_build_no_pre() {
        let version = "1.0.0+100";
        let version = Version::parse(version).unwrap();

        assert_eq!(version,
                   Version {
                       major: 1,
                       minor: 0,
                       patch: 0,
                       pre: None,
                       build: Some(String::from("100")),
                   });
    }
}
