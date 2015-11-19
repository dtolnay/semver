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

/// parse a word
fn word(i: &[u8]) -> IResult<&[u8], String> {
    map_res!(i,
             nom::alphanumeric,
             |d: &[u8]| String::from_utf8(d.to_vec()))
}

/// parse a . and then a u32
named!(dot_number<&[u8], u32>, preceded!(char!('.'), number));

named!(pre<&[u8], Option<String> >, opt!(complete!(preceded!(char!('-'), word))));

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
                patch_pre: opt!(complete!(chain!(
                    patch: dot_number ~
                    pre: pre,
                    || { (patch, pre) }
                ))),
                || {
                    let (patch, pre) = match patch_pre {
                        Some((patch, ref pre)) => (patch, pre.clone()),
                        None => (0, None),
                    };

                    (minor, patch, pre)
                }
        ))),
        || {
            let (minor, patch, pre) = match rest {
                Some((minor, patch, ref pre)) => (minor, patch, pre.clone()),
                None => (0, 0, None),
            };
            super::Version { major: major, minor: minor, patch: patch, pre: pre }
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
            pre: None,
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
            pre: None,
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
            pre: None,
        };

        assert_eq!(version(v1), done(v2));
    }

    #[test]
    fn parse_pre() {
        let v1 = "1.0.0-alpha".as_bytes();
        let v2 = Version {
            major: 1,
            minor: 0,
            patch: 0,
            pre: Some(String::from("alpha")),
        };

        assert_eq!(version(v1), done(v2));
    }
}
