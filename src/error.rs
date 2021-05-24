use crate::parse::Error;
use std::fmt::{self, Debug, Display};

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let _ = formatter;
        unimplemented!()
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let _ = formatter;
        unimplemented!()
    }
}
