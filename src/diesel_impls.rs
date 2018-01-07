use diesel::backend::Backend;
use diesel::types::{FromSql, ToSql, IsNull, ToSqlOutput, Text};
use std::io::Write;
use std::error::Error;

use {Version, VersionReq};

impl<DB> FromSql<Text, DB> for Version
where
    DB: Backend,
    *const str: FromSql<Text, DB>,
{
    fn from_sql(input: Option<&DB::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
        let str_ptr = <*const str as FromSql<Text, DB>>::from_sql(input)?;
        let s = unsafe { &*str_ptr };
        s.parse().map_err(Into::into)
    }
}

impl<DB: Backend> ToSql<Text, DB> for Version {
    fn to_sql<W: Write>(&self, out: &mut ToSqlOutput<W, DB>) -> Result<IsNull, Box<Error + Send + Sync>> {
        write!(out, "{}", self)?;
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for VersionReq
where
    DB: Backend,
    *const str: FromSql<Text, DB>,
{
    fn from_sql(input: Option<&DB::RawValue>) -> Result<Self, Box<Error + Send + Sync>> {
        let str_ptr = <*const str as FromSql<Text, DB>>::from_sql(input)?;
        let s = unsafe { &*str_ptr };
        s.parse().map_err(Into::into)
    }
}

impl<DB: Backend> ToSql<Text, DB> for VersionReq {
    fn to_sql<W: Write>(&self, out: &mut ToSqlOutput<W, DB>) -> Result<IsNull, Box<Error + Send + Sync>> {
        write!(out, "{}", self)?;
        Ok(IsNull::No)
    }
}
