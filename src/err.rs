use std::fmt::Display;

use chumsky::error::Simple;
use snafu::prelude::*;
use snafu::Snafu;

#[derive(Debug, PartialEq, Snafu)]
#[snafu(visibility(pub))]
pub enum ParseError<'a, T: Display> {
    #[snafu(display(
        "Parser internals has returned errors. This either due to invalid input or a bug. {:#?}",
        errors
    ))]
    InternalErr { errors: Vec<Simple<'a, char>> },
    #[snafu(display("Invalid input. Got {}", inp))]
    InvalidInput { inp: T },
}
