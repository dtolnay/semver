use crate::error::{ErrorKind, Position};
use crate::identifier::Identifier;
use crate::{BuildMetadata, Comparator, Prerelease, Version, VersionReq};
use std::str::FromStr;

pub struct Error {
    pub(crate) kind: ErrorKind,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut pos = Position::Major;
        let (major, text) = numeric_identifier(text, pos)?;
        let text = dot(text, pos)?;

        pos = Position::Minor;
        let (minor, text) = numeric_identifier(text, pos)?;
        let text = dot(text, pos)?;

        pos = Position::Patch;
        let (patch, text) = numeric_identifier(text, pos)?;

        if text.is_empty() {
            return Ok(Version::new(major, minor, patch));
        }

        let (pre, text) = if let Some(text) = text.strip_prefix('-') {
            pos = Position::Pre;
            let (pre, text) = prerelease_identifier(text)?;
            if pre.is_empty() {
                return Err(Error::new(ErrorKind::EmptySegment(pos)));
            }
            (pre, text)
        } else {
            (Prerelease::EMPTY, text)
        };

        let (build, text) = if let Some(text) = text.strip_prefix('+') {
            pos = Position::Build;
            let (build, text) = build_identifier(text)?;
            if build.is_empty() {
                return Err(Error::new(ErrorKind::EmptySegment(pos)));
            }
            (build, text)
        } else {
            (BuildMetadata::EMPTY, text)
        };

        if let Some(unexpected) = text.chars().next() {
            return Err(Error::new(ErrorKind::UnexpectedCharAfter(pos, unexpected)));
        }

        Ok(Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }
}

impl FromStr for VersionReq {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for Comparator {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let _ = text;
        unimplemented!()
    }
}

impl FromStr for Prerelease {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (pre, rest) = prerelease_identifier(text)?;
        if !rest.is_empty() {
            return Err(Error::new(ErrorKind::IllegalCharacter(Position::Pre)));
        }
        Ok(pre)
    }
}

impl FromStr for BuildMetadata {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (build, rest) = build_identifier(text)?;
        if !rest.is_empty() {
            return Err(Error::new(ErrorKind::IllegalCharacter(Position::Build)));
        }
        Ok(build)
    }
}

impl Error {
    fn new(kind: ErrorKind) -> Self {
        Error { kind }
    }
}

fn numeric_identifier(input: &str, pos: Position) -> Result<(u64, &str), Error> {
    let mut len = 0;
    let mut value = 0u64;

    while let Some(&digit) = input.as_bytes().get(len) {
        if digit < b'0' || digit > b'9' {
            break;
        }
        if value == 0 && len > 0 {
            return Err(Error::new(ErrorKind::LeadingZero(pos)));
        }
        match value
            .checked_mul(10)
            .and_then(|value| value.checked_add((digit - b'0') as u64))
        {
            Some(sum) => value = sum,
            None => return Err(Error::new(ErrorKind::Overflow(pos))),
        }
        len += 1;
    }

    if len > 0 {
        Ok((value, &input[len..]))
    } else if let Some(unexpected) = input[len..].chars().next() {
        Err(Error::new(ErrorKind::UnexpectedChar(pos, unexpected)))
    } else {
        Err(Error::new(ErrorKind::UnexpectedEnd(pos)))
    }
}

fn dot(input: &str, pos: Position) -> Result<&str, Error> {
    if let Some(rest) = input.strip_prefix('.') {
        Ok(rest)
    } else if let Some(unexpected) = input.chars().next() {
        Err(Error::new(ErrorKind::UnexpectedCharAfter(pos, unexpected)))
    } else {
        Err(Error::new(ErrorKind::UnexpectedEnd(pos)))
    }
}

fn prerelease_identifier(input: &str) -> Result<(Prerelease, &str), Error> {
    let (string, rest) = identifier(input, Position::Pre)?;
    let identifier = unsafe { Identifier::new_unchecked(string) };
    Ok((Prerelease { identifier }, rest))
}

fn build_identifier(input: &str) -> Result<(BuildMetadata, &str), Error> {
    let (string, rest) = identifier(input, Position::Build)?;
    let identifier = unsafe { Identifier::new_unchecked(string) };
    Ok((BuildMetadata { identifier }, rest))
}

fn identifier(input: &str, pos: Position) -> Result<(&str, &str), Error> {
    let mut accumulated_len = 0;
    let mut segment_len = 0;
    let mut segment_has_nondigit = false;

    loop {
        match input.as_bytes().get(accumulated_len + segment_len) {
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') | Some(b'-') => {
                segment_len += 1;
                segment_has_nondigit = true;
            }
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            boundary => {
                if segment_len == 0 {
                    if accumulated_len == 0 && boundary != Some(&b'.') {
                        return Ok(("", input));
                    } else {
                        return Err(Error::new(ErrorKind::EmptySegment(pos)));
                    }
                }
                if pos == Position::Pre
                    && segment_len > 1
                    && !segment_has_nondigit
                    && input[accumulated_len..].starts_with('0')
                {
                    return Err(Error::new(ErrorKind::LeadingZero(pos)));
                }
                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                    segment_has_nondigit = false;
                } else {
                    return Ok(input.split_at(accumulated_len));
                }
            }
        }
    }
}
