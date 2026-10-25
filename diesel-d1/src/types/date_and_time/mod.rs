//! Code derived from Diesel project, see LICENSE for details.

use diesel::{
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types,
};

use crate::{backend::D1Backend, types::TimestamptzSqlite, value::D1Value};

type Sqlite = D1Backend;
type SqliteValue<'a, 'b, 'c> = D1Value;

#[cfg(feature = "chrono")]
mod chrono;

impl FromSql<sql_types::Date, Sqlite> for String {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<sql_types::Date, Sqlite> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<sql_types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<sql_types::Date, Sqlite> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<sql_types::Date, Sqlite>>::to_sql(self as &str, out)
    }
}

impl FromSql<sql_types::Time, Sqlite> for String {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<sql_types::Time, Sqlite> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<sql_types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<sql_types::Time, Sqlite> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<sql_types::Time, Sqlite>>::to_sql(self as &str, out)
    }
}

impl FromSql<sql_types::Timestamp, Sqlite> for String {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<sql_types::Timestamp, Sqlite> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<sql_types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<sql_types::Timestamp, Sqlite> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<sql_types::Timestamp, Sqlite>>::to_sql(self as &str, out)
    }
}

impl FromSql<TimestamptzSqlite, Sqlite> for String {
    fn from_sql(value: SqliteValue<'_, '_, '_>) -> deserialize::Result<Self> {
        FromSql::<sql_types::Text, Sqlite>::from_sql(value)
    }
}

impl ToSql<TimestamptzSqlite, Sqlite> for str {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<sql_types::Text, Sqlite>::to_sql(self, out)
    }
}

impl ToSql<TimestamptzSqlite, Sqlite> for String {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<TimestamptzSqlite, Sqlite>>::to_sql(self as &str, out)
    }
}
