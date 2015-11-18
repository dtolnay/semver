#[macro_use]
extern crate nom;

pub mod parser;

use std::result;

#[derive(PartialEq,Debug)]
struct Version {
    major: u32,
}

#[derive(Debug)]
enum SemVerError {
    GenericError,
}

type Result<T> = result::Result<T, SemVerError>;

impl From<()> for SemVerError {
    fn from(_: ()) -> SemVerError {
        SemVerError::GenericError
    }
}

impl Version {
    pub fn parse(version: &str) -> Result<Version> {
        let major = try!(parser::try_number(version.as_bytes()));

        Ok(Version { major: major })
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn parse_major_number() {
        let version = "10";
        let version = Version::parse(version).unwrap();

        assert_eq!(version, Version { major: 10 });
    }
}
