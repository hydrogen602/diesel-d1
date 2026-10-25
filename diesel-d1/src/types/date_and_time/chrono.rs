//! Code derived from Diesel project, see LICENSE for details.

extern crate chrono;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{Date, Time, Timestamp},
};

use crate::{backend::D1Backend, types::TimestamptzSqlite, utils::D1Error};

use self::chrono::{DateTime, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

type Sqlite = D1Backend;

/// Warning to future editors:
/// Changes in the following formats need to be kept in sync
/// with the formats of the ["time"](super::time) module.
/// We do not need a distinction between whole second and
/// subsecond since %.f will only print the dot if needed.
/// We always print as many subsecond as his given to us,
/// this means the subsecond part can be 3, 6 or 9 digits.
const DATE_FORMAT: &str = "%F";

const ENCODE_TIME_FORMAT: &str = "%T%.f";

const TIME_FORMATS: [&str; 9] = [
    // Most likely formats
    "%T%.f", "%T", // All other valid formats in order of increasing specificity
    "%R", "%RZ", "%R%:z", "%TZ", "%T%:z", "%T%.fZ", "%T%.f%:z",
];

const ENCODE_NAIVE_DATETIME_FORMAT: &str = "%F %T%.f";

const ENCODE_DATETIME_FORMAT: &str = "%F %T%.f%:z";

const NAIVE_DATETIME_FORMATS: [&str; 18] = [
    // Most likely formats
    "%F %T%.f",
    "%F %T%.f%:z",
    "%F %T",
    "%F %T%:z",
    // All other formats in order of increasing specificity
    "%F %R",
    "%F %RZ",
    "%F %R%:z",
    "%F %TZ",
    "%F %T%.fZ",
    "%FT%R",
    "%FT%RZ",
    "%FT%R%:z",
    "%FT%T",
    "%FT%TZ",
    "%FT%T%:z",
    "%FT%T%.f",
    "%FT%T%.fZ",
    "%FT%T%.f%:z",
];

const DATETIME_FORMATS: [&str; 12] = [
    // Most likely formats
    "%F %T%.f%:z",
    "%F %T%:z",
    // All other formats in order of increasing specificity
    "%F %RZ",
    "%F %R%:z",
    "%F %TZ",
    "%F %T%.fZ",
    "%FT%RZ",
    "%FT%R%:z",
    "%FT%TZ",
    "%FT%T%:z",
    "%FT%T%.fZ",
    "%FT%T%.f%:z",
];

fn parse_julian(julian_days: f64) -> Option<NaiveDateTime> {
    const EPOCH_IN_JULIAN_DAYS: f64 = 2_440_587.5;
    const SECONDS_IN_DAY: f64 = 86400.0;
    let timestamp = (julian_days - EPOCH_IN_JULIAN_DAYS) * SECONDS_IN_DAY;
    #[allow(clippy::cast_possible_truncation)] // we want to truncate
    let seconds = timestamp.trunc() as i64;
    // that's not true, `fract` is always > 0
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let nanos = (timestamp.fract() * 1E9) as u32;
    #[allow(deprecated)] // otherwise we would need to bump our minimal chrono version
    NaiveDateTime::from_timestamp_opt(seconds, nanos)
}

impl FromSql<Date, Sqlite> for NaiveDate {
    fn from_sql(mut value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        value
            .parse_string(|s| Self::parse_from_str(s, DATE_FORMAT))
            .ok_or_else(|| D1Error::new("Expected text"))?
            .map_err(Into::into)
    }
}

impl ToSql<Date, Sqlite> for NaiveDate {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.format(DATE_FORMAT).to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Time, Sqlite> for NaiveTime {
    fn from_sql(mut value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        value
            .parse_string(|text| {
                for format in TIME_FORMATS {
                    if let Ok(time) = Self::parse_from_str(text, format) {
                        return Ok(time);
                    }
                }

                Err(format!("Invalid time {text}").into())
            })
            .ok_or_else(|| D1Error::new("Expected text"))?
    }
}

impl ToSql<Time, Sqlite> for NaiveTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.format(ENCODE_TIME_FORMAT).to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Timestamp, Sqlite> for NaiveDateTime {
    fn from_sql(mut value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        value
            .parse_string(|text| {
                for format in NAIVE_DATETIME_FORMATS {
                    if let Ok(dt) = Self::parse_from_str(text, format) {
                        return Ok(dt);
                    }
                }

                if let Ok(julian_days) = text.parse::<f64>()
                    && let Some(timestamp) = parse_julian(julian_days)
                {
                    return Ok(timestamp);
                }

                Err(format!("Invalid datetime {text}").into())
            })
            .ok_or_else(|| D1Error::new("Expected text"))?
    }
}

impl ToSql<Timestamp, Sqlite> for NaiveDateTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.format(ENCODE_NAIVE_DATETIME_FORMAT).to_string());
        Ok(IsNull::No)
    }
}

// A naive datetime carries no offset, so it reads and writes as a plain `Timestamp`.

impl FromSql<TimestamptzSqlite, Sqlite> for NaiveDateTime {
    fn from_sql(value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        FromSql::<Timestamp, Sqlite>::from_sql(value)
    }
}

impl ToSql<TimestamptzSqlite, Sqlite> for NaiveDateTime {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        ToSql::<Timestamp, Sqlite>::to_sql(self, out)
    }
}

impl FromSql<TimestamptzSqlite, Sqlite> for DateTime<Utc> {
    fn from_sql(mut value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        // First try to parse the timezone
        if let Ok(dt) = value
            .parse_string(|text| {
                for format in DATETIME_FORMATS {
                    if let Ok(dt) = DateTime::parse_from_str(text, format) {
                        return Ok(dt.with_timezone(&Utc));
                    }
                }

                Err(())
            })
            .ok_or_else(|| D1Error::new("Expected text"))?
        {
            return Ok(dt);
        }

        // Fallback on assuming Utc
        let naive_date_time =
            <NaiveDateTime as FromSql<TimestamptzSqlite, Sqlite>>::from_sql(value)?;
        Ok(Utc.from_utc_datetime(&naive_date_time))
    }
}

impl FromSql<TimestamptzSqlite, Sqlite> for DateTime<Local> {
    fn from_sql(mut value: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        // First try to parse the timezone
        if let Ok(dt) = value
            .parse_string(|text| {
                for format in DATETIME_FORMATS {
                    if let Ok(dt) = DateTime::parse_from_str(text, format) {
                        return Ok(dt.with_timezone(&Local));
                    }
                }

                Err(())
            })
            .ok_or_else(|| D1Error::new("Expected text"))?
        {
            return Ok(dt);
        }

        // Fallback on assuming Local
        let naive_date_time =
            <NaiveDateTime as FromSql<TimestamptzSqlite, Sqlite>>::from_sql(value)?;
        Ok(Local::from_utc_datetime(&Local, &naive_date_time))
    }
}

impl<TZ: TimeZone> ToSql<TimestamptzSqlite, Sqlite> for DateTime<TZ> {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        // Converting to UTC ensures consistency
        let dt_utc = self.with_timezone(&Utc);
        out.set_value(dt_utc.format(ENCODE_DATETIME_FORMAT).to_string());
        Ok(IsNull::No)
    }
}
