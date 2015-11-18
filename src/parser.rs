use std::str;
use nom;
use nom::IResult;

pub fn try_parse(i: &[u8]) -> Result<super::Version, ()> {
    match version(i) {
        IResult::Done(_, d) => Ok(d),
        _ => Err(()),
    }
}

named!(pub version <&[u8], super::Version>, chain!(
        major: number ~
        tag!(".") ~
        minor: number ~
        tag!(".") ~
        patch: number,
        || { super::Version { major: major, minor: minor, patch: patch }
        }
    )
);

fn number(i: &[u8]) -> IResult<&[u8], u32> {
    map_res!(i,
             nom::digit,
             |d| str::FromStr::from_str(str::from_utf8(d).unwrap()))
}

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
