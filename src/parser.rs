use std::str;
use nom;
use nom::IResult;

use version::Identifier;
use version_req::VersionReq;

/// Try to parse a version
///
/// If there's an error, then you just get (). for now.
pub fn try_parse(i: &[u8]) -> Result<super::Version, String> {
    match version(i) {
        IResult::Done(rest, version) => {
            if rest.len() > 0 {
                let err = format!("Failed with unparsed input: '{}'",
                                  String::from_utf8(rest.to_vec()).unwrap());
                Err(err)
            } else{
                Ok(version)
            }
        },
        _ => Err("Parse error".to_string()),
    }
}

pub fn try_parse_req(i: &[u8]) -> Result<VersionReq, String> {
    match version_req(i) {
        IResult::Done(rest, version) => {
            if rest.len() > 0 {
                let err = format!("Failed with unparsed input: '{}'",
                                  String::from_utf8(rest.to_vec()).unwrap());
                Err(err)
            } else{
                Ok(version)
            }
        },
        _ => Err("Parse error".to_string()),
    }
}

named!(version_req<&[u8], VersionReq>, chain!(
        major: number ~
        minor: dot_number ~
        patch: dot_number ~
        extras: extras,
        || {
            let (_, _, _, _) = (major, minor, patch, extras);
            VersionReq {
                sets: Vec::new(),
            }
        }
));

/// parse a u64
fn number(i: &[u8]) -> IResult<&[u8], u64> {
    use std::str::from_utf8;
    map_res!(i,
             map_res!(nom::digit, from_utf8),
             |d| str::FromStr::from_str(d))
}

/// Parse an alphanumeric or a dot ("[0-9A-Za-z.]" in regex)
fn ascii_or_hyphen(chr: u8) -> bool {
    // hyphen
    chr == 45 ||
    // 0-9
    (chr >= 48 && chr <= 57) ||
    // A-Z
    (chr >= 65 && chr <= 90) ||
    // a-z
    (chr >= 97 && chr <= 122)
}

named!(take_ascii_or_hyphen, take_while!(ascii_or_hyphen));

fn convert_identifiers(identifiers: Vec<&str>) -> Vec<Identifier> {
    let mut result = Vec::new();

    for identifier in identifiers {
       match identifier.parse() {
           Ok(n)  => result.push(Identifier::Numeric(n)),
           Err(_) => result.push(Identifier::AlphaNumeric(identifier.to_string())),
       }
    }

    result
}

/// Parse an identifier
fn identifiers(i: &[u8]) -> IResult<&[u8], Vec<Identifier>> {
    use std::str::from_utf8;
    map!(i,
         separated_list!(tag!("."), map_res!(take_ascii_or_hyphen, from_utf8)),
         convert_identifiers)
}

/// parse a . and then a u32
named!(dot_number<&[u8], u64>, preceded!(char!('.'), number));

named!(pre<&[u8], Option<Vec<Identifier> > >,   opt!(complete!(preceded!(tag!("-"), identifiers))));
named!(build<&[u8], Option<Vec<Identifier> > >, opt!(complete!(preceded!(tag!("+"), identifiers))));

named!(extras<&[u8], (Option<Vec<Identifier>>, Option<Vec<Identifier>>) >, pair!(pre, build));

/// parse a version
///
/// A version is currently:
///
/// - a major version number
/// - optionally followed by a dot and a minor version number
/// - optionally followed by a dot and a patch version number
///
/// If some of the versions aren't present, gives a zero.
named!(version<&[u8], super::Version>, chain!(
        major: number ~
        minor: dot_number ~
        patch: dot_number ~
        extras: extras,
        || {
            super::Version {
                major: major,
                minor: minor,
                patch: patch,
                pre: extras.0.clone().unwrap_or(Vec::new()),
                build: extras.1.clone().unwrap_or(Vec::new()),
            }
        }
));


#[cfg(test)]
mod tests {
    use super::number;
    use super::dot_number;
    use super::version;
    use version::Identifier;
    use Version;

    fn done<T>(x: T) -> ::nom::IResult<&'static [u8], T> {
        ::nom::IResult::Done(&[][..], x)
    }

    #[test]
    fn parse_number() {
        let v = "10".as_bytes();

        assert_eq!(number(v), done(10));
    }

    #[test]
    fn parse_dot_number() {
        let v = ".10".as_bytes();

        assert_eq!(dot_number(v), done(10));
    }

    #[test]
    fn parse_version() {
        let v1 = "10.11.12".as_bytes();
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 12,
            pre: Vec::new(),
            build: Vec::new(),
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_pre_basic() {
        let v1 = "1.0.0-alpha".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: vec![Identifier::AlphaNumeric(String::from("alpha"))],
            build: Vec::new(),
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_pre_dot() {
        let v1 = "1.0.0-alpha.1".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: vec![Identifier::AlphaNumeric(String::from("alpha")), Identifier::Numeric(1)],
            build: Vec::new(),
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_build_basic() {
        let v1 = "1.0.0-alpha+001".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: vec![Identifier::AlphaNumeric(String::from("alpha"))],
            build: vec![Identifier::Numeric(1)],
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_build_no_pre() {
        let v1 = "1.0.0+001".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Vec::new(),
            build: vec![Identifier::Numeric(1)],
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_hypen_in_pre() {
        let v1 = "3.0.0-rc1-1".as_bytes();
        let v2 = Version {
            major: 3,
            minor: 0,
            patch: 0,
            pre: vec![Identifier::AlphaNumeric(String::from("rc1-1"))],
            build: Vec::new(),
        };

        assert_eq!(version(v1), done(v2));
    }
}
