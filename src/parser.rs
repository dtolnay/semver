use std::str;
use nom;
use nom::IResult;

pub fn try_parse(i: &[u8]) -> Result<super::Version, ()> {
    match version(i) {
        IResult::Done(_, d) => Ok(d),
        _ => Err(()),
    }
}

fn number(i: &[u8]) -> IResult<&[u8], u32> {
    map_res!(i,
             nom::digit,
             |d| str::FromStr::from_str(str::from_utf8(d).unwrap()))
}

named!(dot_number<&[u8], u32>, chain!(
        tag!(".") ~
        i: number, || { i }
));

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
    use super::version;
    use nom::IResult::Done;
    use Version;

    #[test]
    fn parse_number() {
        let v = "10";

        assert_eq!(number(v.as_bytes()), Done(&[][..], 10));
    }

    #[test]
    fn parse_major() {
        let v1 = "10";
        let v2 = Version {
            major: 10,
            minor: 0,
            patch: 0,
        };

        assert_eq!(version(v1.as_bytes()), Done(&[][..], v2));
    }

    #[test]
    fn parse_minor() {
        let v1 = "10.11";
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 0,
        };

        assert_eq!(version(v1.as_bytes()), Done(&[][..], v2));
    }

    #[test]
    fn parse_version() {
        let v1 = "10.11.12";
        let v2 = Version {
            major: 10,
            minor: 11,
            patch: 12,
        };

        assert_eq!(version(v1.as_bytes()), Done(&[][..], v2));
    }
}
