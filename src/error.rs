use crate::parse::Error;
use std::fmt::{self, Debug, Display};

pub(crate) enum ErrorKind {
    UnexpectedEnd(Position),
    UnexpectedChar(Position, char),
    UnexpectedCharAfter(Position, char),
    ExpectedCommaFound(Position, char),
    LeadingZero(Position),
    Overflow(Position),
    EmptySegment(Position),
    IllegalCharacter(Position),
    UnexpectedAfterWildcard,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Position {
    Major,
    Minor,
    Patch,
    Pre,
    Build,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedEnd(pos) => {
                formatter.write_str("unexpected end of input while parsing ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::UnexpectedChar(pos, ch) => {
                formatter.write_str("unexpected character ")?;
                Debug::fmt(ch, formatter)?;
                formatter.write_str(" while parsing ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::UnexpectedCharAfter(pos, ch) => {
                formatter.write_str("unexpected character ")?;
                Debug::fmt(ch, formatter)?;
                formatter.write_str(" after ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::ExpectedCommaFound(pos, ch) => {
                formatter.write_str("expected comma after ")?;
                Display::fmt(pos, formatter)?;
                formatter.write_str(", found ")?;
                Debug::fmt(ch, formatter)?;
                Ok(())
            }
            ErrorKind::LeadingZero(pos) => {
                formatter.write_str("invalid leading zero in ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::Overflow(pos) => {
                formatter.write_str("value of ")?;
                Display::fmt(pos, formatter)?;
                formatter.write_str(" exceeds u64::MAX")?;
                Ok(())
            }
            ErrorKind::EmptySegment(pos) => {
                formatter.write_str("empty identifier segment in ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::IllegalCharacter(pos) => {
                formatter.write_str("unexpected character in ")?;
                Display::fmt(pos, formatter)?;
                Ok(())
            }
            ErrorKind::UnexpectedAfterWildcard => {
                formatter.write_str("unexpected character after wildcard in version req")
            }
        }
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(match self {
            Position::Major => "major version number",
            Position::Minor => "minor version number",
            Position::Patch => "patch version number",
            Position::Pre => "pre-release identifier",
            Position::Build => "build metadata",
        })
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Error(\"")?;
        Display::fmt(self, formatter)?;
        formatter.write_str("\")")?;
        Ok(())
    }
}
