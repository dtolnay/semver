use std::str;
use nom;
use nom::IResult;

pub fn try_number(i: &[u8]) -> Result<u32, ()> {
    match number(i) {
        IResult::Done(_, d) => Ok(d),
        _ => Err(()),
    }
}

fn number(i: &[u8]) -> IResult<&[u8], u32> {
    map_res!(i, nom::digit, |d| { str::FromStr::from_str(str::from_utf8(d).unwrap()) })
}

#[cfg(test)]
mod tests {
    use super::number;
    use nom::IResult::Done;

    #[test]
    fn parse_number() {
        let version = "10";

        assert_eq!(number(version.as_bytes()), Done(&[][..], 10));
    }
}
