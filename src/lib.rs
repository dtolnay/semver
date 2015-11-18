#[macro_use]
extern crate nom;

pub mod parser;

use std::result;

#[derive(PartialEq,Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

#[derive(Debug)]
enum SemVerError {
    GenericError,
}

pub type Result<T> = result::Result<T, SemVerError>;

impl From<()> for SemVerError {
    fn from(_: ()) -> SemVerError {
        SemVerError::GenericError
    }
}

impl Version {
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
                   });
    }
}
