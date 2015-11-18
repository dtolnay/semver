use std::str;
use nom;
use nom::IResult;

/// Try to parse a version
///
/// If there's an error, then you just get (). for now.
pub fn try_parse(i: &[u8]) -> Result<super::Version, ()> {
    match version(i) {
        IResult::Done(_, d) => Ok(d),
        _ => Err(()),
    }
}

/// parse a u32
fn number(i: &[u8]) -> IResult<&[u8], u32> {
    map_res!(i,
             nom::digit,
             |d| str::FromStr::from_str(str::from_utf8(d).unwrap()))
}

/// parse a . and then a u32
named!(dot_number<&[u8], u32>, chain!(
        tag!(".") ~
        i: number, || { i }
));

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
        rest: opt!(complete!(chain!(
                minor: dot_number ~
                patch: opt!(complete!(dot_number)),
                || { (minor, patch.unwrap_or(0)) }
        ))),
        || {
            let (minor, patch) = rest.unwrap_or((0, 0));
            super::Version { major: major, minor: minor, patch: patch }
        }
));


#[cfg(test)]
mod tests {
    use super::number;
    use super::dot_number;
    use super::version;
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
    fn parse_major() {
        let v1 = "10".as_bytes();
        let v2 = Version {
            major: 10,
            minor: 0,
            patch: 0,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_minor() {
        let v1 = "10.11".as_bytes();
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 0,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_version() {
        let v1 = "10.11.12".as_bytes();
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 12,
        };

        assert_eq!(version(v1), done(v2));
    }
}
