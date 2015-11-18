#[macro_use]
extern crate nom;

pub mod parser;

use nom::IResult;

#[derive(PartialEq,Debug)]
struct Version {
    major: u32,
}

impl Version {
    pub fn parse(version: &str) -> Version {
        let major = match parser::number(version.as_bytes()) {
            IResult::Done(_, o) => o,
            _ => panic!("not yet"),
        };

        Version { major: major }
    }
}

#[cfg(test)]
mod tests {
    use super::Version;

    #[test]
    fn parse_major_number() {
        let version = "10";
        let version = Version::parse(version);

        assert_eq!(version, Version { major: 10 });
    }
}
