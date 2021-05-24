use crate::parse::Error;
use std::fmt::{self, Debug, Display};

pub(crate) enum ErrorKind {
    LeadingZero(Position),
    EmptySegment(Position),
    IllegalCharacter(Position),
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Position {
    Pre,
    Build,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::LeadingZero(pos) => {
                formatter.write_str("invalid leading zero in ")?;
                Display::fmt(pos, formatter)?;
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
        }
    }
}

impl Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(match self {
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
