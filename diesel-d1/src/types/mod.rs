use diesel::{
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{self, HasSqlType},
};

use crate::{
    backend::{D1Backend, D1TypeName},
    utils::D1Error,
    value::{D1Value, IntError, exceeds_js_safe_integer},
};

// Boolean
impl HasSqlType<sql_types::Bool> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Integer
    }
}

impl FromSql<sql_types::Bool, D1Backend> for bool {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        if let Some(bool_number) = value.read_number()
            && (bool_number == 0.0 || bool_number == 1.0)
        {
            Ok(bool_number != 0.0)
        } else {
            Err(D1Error {
                message: format!("expected bool but got: {}", value),
            }
            .into())
        }
    }
}

impl ToSql<sql_types::Bool, D1Backend> for bool {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

// SMALL INT

impl HasSqlType<sql_types::SmallInt> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Integer
    }
}

impl FromSql<sql_types::SmallInt, D1Backend> for i16 {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        let Some(text) = value.read_number() else {
            return Err(D1Error {
                message: format!("expected small int but got: {}", value),
            }
            .into());
        };
        Ok(text as i16)
    }
}

impl ToSql<sql_types::SmallInt, D1Backend> for i16 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(i32::from(*self));
        Ok(IsNull::No)
    }
}

// ------

// Int

impl HasSqlType<sql_types::Integer> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Integer
    }
}

impl FromSql<sql_types::Integer, D1Backend> for i32 {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        let text = value.read_integer()?;
        Ok(text.try_into()?)
    }
}

impl ToSql<sql_types::Integer, D1Backend> for i32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

// ------

// BigInt is not supported by D1

/// Note:
///   D1 supports 64-bit signed INTEGER values internally,
///   however BigInts are not currently supported in the API yet.
///   JavaScript integers are safe up to Number.MAX_SAFE_INTEGER.
///   See: https://developers.cloudflare.com/d1/worker-api/
impl HasSqlType<sql_types::BigInt> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Integer
    }
}

impl FromSql<sql_types::BigInt, D1Backend> for i64 {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        value.read_integer().map_err(From::from)
    }
}

impl ToSql<sql_types::BigInt, D1Backend> for i64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        if exceeds_js_safe_integer(*self) {
            return Err(IntError::NotASafeInteger(*self).into());
        }
        out.set_value(*self as f64);
        Ok(IsNull::No)
    }
}

// ------

// Float

impl HasSqlType<sql_types::Float> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Real
    }
}

impl FromSql<sql_types::Float, D1Backend> for f32 {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        let text = value.read_number().ok_or_else(|| D1Error {
            message: format!("expected float but got: {}", value),
        })?;
        Ok(text as f32)
    }
}

impl ToSql<sql_types::Float, D1Backend> for f32 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(f64::from(*self));
        Ok(IsNull::No)
    }
}

// ------

// Double

impl HasSqlType<sql_types::Double> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Real
    }
}

impl FromSql<sql_types::Double, D1Backend> for f64 {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        let text = value.read_number().ok_or_else(|| D1Error {
            message: format!("expected double but got: {}", value),
        })?;
        Ok(text)
    }
}

impl ToSql<sql_types::Double, D1Backend> for f64 {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}

// ------

// Text

impl HasSqlType<sql_types::Text> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Text
    }
}

impl FromSql<sql_types::Text, D1Backend> for String {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        let text = value.read_string().ok_or_else(|| D1Error {
            message: format!("expected text but got: {}", value),
        })?;
        Ok(text)
    }
}

impl ToSql<sql_types::Text, D1Backend> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(self);
        Ok(IsNull::No)
    }
}

// ------

// Blob

impl HasSqlType<sql_types::Binary> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Blob
    }
}

impl FromSql<sql_types::Binary, D1Backend> for Vec<u8> {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        value.read_blob()
    }
}

impl ToSql<sql_types::Binary, D1Backend> for &[u8] {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(*self);
        Ok(IsNull::No)
    }
}
impl ToSql<sql_types::Binary, D1Backend> for Vec<u8> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        out.set_value(self.as_slice());
        Ok(IsNull::No)
    }
}

// ------ Time related (simplified to only text)

impl HasSqlType<sql_types::Date> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Text
    }
}

impl HasSqlType<sql_types::Time> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Text
    }
}

impl HasSqlType<sql_types::Timestamp> for D1Backend {
    fn metadata(_lookup: &mut ()) -> D1TypeName {
        D1TypeName::Text
    }
}

impl FromSql<sql_types::Date, D1Backend> for String {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, D1Backend>::from_sql(value)
    }
}

impl ToSql<sql_types::Date, D1Backend> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        ToSql::<sql_types::Text, D1Backend>::to_sql(self, out)
    }
}

impl FromSql<sql_types::Time, D1Backend> for String {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, D1Backend>::from_sql(value)
    }
}

impl FromSql<sql_types::Timestamp, D1Backend> for String {
    fn from_sql(value: D1Value) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, D1Backend>::from_sql(value)
    }
}

impl ToSql<sql_types::Timestamp, D1Backend> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, D1Backend>) -> serialize::Result {
        ToSql::<sql_types::Text, D1Backend>::to_sql(self, out)
    }
}
