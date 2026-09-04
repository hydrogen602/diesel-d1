use diesel::prelude::*;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use diesel_d1::D1ConnectionBuilder;
use worker::*;

use crate::{D1_NAME, D1Connection};

mod sample_schema {
    diesel::table! {
        js_limits (id) {
            id -> Integer,
            int_val -> BigInt,
            real_val -> Double,
        }
    }
}

const MAX_SAFE_INTEGER: i64 = 9007199254740991;
const UNSAFE_POS: i64 = MAX_SAFE_INTEGER + 1;
const UNSAFE_NEG: i64 = -MAX_SAFE_INTEGER - 1;
const LARGE_REAL: f64 = 1.0e20;
const REAL_AT_UNSAFE_INT: f64 = 9007199254740992.0;

#[derive(Queryable, Debug)]
struct JsLimitRow {
    id: i32,
    int_val: i64,
    real_val: f64,
}

fn assert_safe_integer_error(err: Error, expected_kind: &str) {
    let (kind, inner) = match err {
        Error::SerializationError(inner) => ("SerializationError", inner),
        Error::DeserializationError(inner) => ("DeserializationError", inner),
        other => panic!("expected {expected_kind}, got {other:?}"),
    };
    assert_eq!(kind, expected_kind);
    let message = inner.to_string();
    assert!(
        message.contains("MAX_SAFE_INTEGER"),
        "unexpected {expected_kind}: {message}"
    );
}

async fn load_ids(d1: &mut D1Connection) -> Vec<i32> {
    sample_schema::js_limits::table
        .select(sample_schema::js_limits::id)
        .order(sample_schema::js_limits::id)
        .load(d1)
        .await
        .unwrap()
}

pub async fn test_js_safe_integer_limits(env: &Env) {
    let mut d1 = D1ConnectionBuilder::new()
        .env(env)
        .name(D1_NAME)
        .build()
        .unwrap();

    let safe_rows: Vec<JsLimitRow> = sample_schema::js_limits::table
        .select(sample_schema::js_limits::all_columns)
        .filter(sample_schema::js_limits::id.eq_any(vec![1, 2]))
        .order(sample_schema::js_limits::id)
        .load(&mut d1)
        .await
        .unwrap();

    assert_eq!(safe_rows.len(), 2);
    assert_eq!(safe_rows[0].id, 1);
    assert_eq!(safe_rows[0].int_val, MAX_SAFE_INTEGER);
    assert_eq!(safe_rows[0].real_val, LARGE_REAL);
    assert_eq!(safe_rows[1].id, 2);
    assert_eq!(safe_rows[1].int_val, -MAX_SAFE_INTEGER);
    assert_eq!(safe_rows[1].real_val, -LARGE_REAL);

    // Test: D1T2 - Integers larger than `Number.MAX_SAFE_INTEGER` are rejected
    let too_large_insert = diesel::insert_into(sample_schema::js_limits::table).values((
        sample_schema::js_limits::id.eq(99),
        sample_schema::js_limits::int_val.eq(UNSAFE_POS),
        sample_schema::js_limits::real_val.eq(0.0),
    ));
    assert_safe_integer_error(
        too_large_insert.execute(&mut d1).await.unwrap_err(),
        "SerializationError",
    );

    // Test: D1T2 - Integers larger than `Number.MAX_SAFE_INTEGER` are rejected
    let too_large_filter = sample_schema::js_limits::table
        .select(sample_schema::js_limits::id)
        .filter(sample_schema::js_limits::int_val.eq(UNSAFE_POS));
    assert_safe_integer_error(
        too_large_filter.load::<i32>(&mut d1).await.unwrap_err(),
        "SerializationError",
    );

    // Test: D1T3 - Integers smaller than `-Number.MAX_SAFE_INTEGER` are rejected
    let too_small_insert = diesel::insert_into(sample_schema::js_limits::table).values((
        sample_schema::js_limits::id.eq(99),
        sample_schema::js_limits::int_val.eq(UNSAFE_NEG),
        sample_schema::js_limits::real_val.eq(0.0),
    ));
    assert_safe_integer_error(
        too_small_insert.execute(&mut d1).await.unwrap_err(),
        "SerializationError",
    );

    // Test: D1T3 - Integers smaller than `-Number.MAX_SAFE_INTEGER` are rejected
    let too_small_filter = sample_schema::js_limits::table
        .select(sample_schema::js_limits::id)
        .filter(sample_schema::js_limits::int_val.eq(UNSAFE_NEG));
    assert_safe_integer_error(
        too_small_filter.load::<i32>(&mut d1).await.unwrap_err(),
        "SerializationError",
    );

    assert_eq!(load_ids(&mut d1).await, vec![1, 2, 3, 4]);

    // Test: D1T2 - Integers larger than `Number.MAX_SAFE_INTEGER` are rejected
    // Test: D1T4 - Unsafe integers fail to decode
    let too_large_decode = sample_schema::js_limits::table
        .select(sample_schema::js_limits::int_val)
        .filter(sample_schema::js_limits::id.eq(3));
    assert_safe_integer_error(
        too_large_decode.load::<i64>(&mut d1).await.unwrap_err(),
        "DeserializationError",
    );

    // Test: D1T3 - Integers smaller than `-Number.MAX_SAFE_INTEGER` are rejected
    // Test: D1T4 - Unsafe integers fail to decode
    let too_small_decode = sample_schema::js_limits::table
        .select(sample_schema::js_limits::int_val)
        .filter(sample_schema::js_limits::id.eq(4));
    assert_safe_integer_error(
        too_small_decode.load::<i64>(&mut d1).await.unwrap_err(),
        "DeserializationError",
    );

    // Test: D1T5 - Reals beyond `Number.MAX_SAFE_INTEGER` succeed
    let reals: Vec<(i32, f64)> = sample_schema::js_limits::table
        .select((
            sample_schema::js_limits::id,
            sample_schema::js_limits::real_val,
        ))
        .order(sample_schema::js_limits::id)
        .load(&mut d1)
        .await
        .unwrap();

    assert_eq!(reals.len(), 4);
    assert_eq!(reals[0], (1, LARGE_REAL));
    assert_eq!(reals[1], (2, -LARGE_REAL));
    assert_eq!(reals[2], (3, REAL_AT_UNSAFE_INT));
    assert_eq!(reals[3], (4, -REAL_AT_UNSAFE_INT));

    // Test: D1T5 - Reals beyond `Number.MAX_SAFE_INTEGER` succeed
    diesel::insert_into(sample_schema::js_limits::table)
        .values((
            sample_schema::js_limits::id.eq(5),
            sample_schema::js_limits::int_val.eq(0i64),
            sample_schema::js_limits::real_val.eq(LARGE_REAL),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    // Test: D1T5 - Reals beyond `Number.MAX_SAFE_INTEGER` succeed
    diesel::insert_into(sample_schema::js_limits::table)
        .values((
            sample_schema::js_limits::id.eq(6),
            sample_schema::js_limits::int_val.eq(0i64),
            sample_schema::js_limits::real_val.eq(-LARGE_REAL),
        ))
        .execute(&mut d1)
        .await
        .unwrap();

    let inserted: Vec<(i32, i64, f64)> = sample_schema::js_limits::table
        .select(sample_schema::js_limits::all_columns)
        .filter(sample_schema::js_limits::id.eq_any(vec![5, 6]))
        .order(sample_schema::js_limits::id)
        .load(&mut d1)
        .await
        .unwrap();

    assert_eq!(inserted.len(), 2);
    assert_eq!(inserted[0], (5, 0, LARGE_REAL));
    assert_eq!(inserted[1], (6, 0, -LARGE_REAL));

    // Test: D1T5 - Reals beyond `Number.MAX_SAFE_INTEGER` succeed
    let matched: Vec<i32> = sample_schema::js_limits::table
        .select(sample_schema::js_limits::id)
        .filter(sample_schema::js_limits::real_val.eq(LARGE_REAL))
        .order(sample_schema::js_limits::id)
        .load(&mut d1)
        .await
        .unwrap();
    assert_eq!(matched, vec![1, 5]);
}
