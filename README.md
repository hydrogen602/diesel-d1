# Diesel D1 backend + connection

This is a custom backend/connection for [Diesel](https://diesel.rs/) for [D1](https://developers.cloudflare.com/d1/) targeting Cloudflare Workers, at the moment.

**IMPORTANT:** THIS IS NOT PRODUCTION READY YET, THINGS WILL PROBABLY BREAK (feel free to use it tho).

## Compatability

At the moment, this only supports Cloudflare Workers via the D1 binding (therefore, it only supports WASM). ~~Generic support for the HTTP API is coming later.~~

# Fork notes

- Primarily modernizing it, focusing on correctness (like passing an i64 > `Number.MAX_SAFE_INTEGER` should be an error as JS will lose precision), and not panicking on errors (TODO).

# Integration testing

## Test Coverage

### Select

| ID    | Category  | Type     | Case                     | Status |
| ----- | --------- | -------- | ------------------------ | ------ |
| S1    | Select    |          | All columns              | ✅     |
| S2    | Select    |          | Specific columns         |        |
| S3    | Select    |          | Tuple of columns         | ✅     |
| SW1   | Where     | null     | `is null`                | ✅     |
| SW2   | Where     | null     | `is not null`            |        |
| SW3   | Where     | int      | `<` int                  | ✅     |
| SW4   | Where     | int      | `>` int                  |        |
| SW5   | Where     | int      | `<=` int                 |        |
| SW6   | Where     | int      | `>=` int                 |        |
| SW7   | Where     | int      | `=` int                  |        |
| SW8   | Where     | int      | `<>` int                 |        |
| SW9   | Where     | int      | `in` int[]               |        |
| SW10  | Where     | int      | `not in` int[]           |        |
| SW11  | Where     | int      | `between` int            |        |
| SW12  | Where     | int      | `not between` int        |        |
| SW13  | Where     | string   | `<` string               |        |
| SW14  | Where     | string   | `>` string               |        |
| SW15  | Where     | string   | `<=` string              |        |
| SW16  | Where     | string   | `>=` string              |        |
| SW17  | Where     | string   | `=` string               |        |
| SW18  | Where     | string   | `<>` string              |        |
| SW19  | Where     | string   | `in` string[]            |        |
| SW20  | Where     | string   | `not in` string[]        |        |
| SW21  | Where     | string   | `like` string            |        |
| SW22  | Where     | string   | `not like` string        |        |
| SW23  | Where     | Compound | `and`                    |        |
| SW24  | Where     | Compound | `or`                     |        |
| SW25  | Where     | Compound | `not`                    |        |
| SO1   | Order by  |          | Single column            |        |
| SO2   | Order by  |          | Multiple columns         |        |
| SO3   | Order by  |          | Direction                |        |
| SL1   | Limit     |          | Single value             |        |
| SOff1 | Offset    |          | Single value             |        |
| SJ1   | Join      |          | Inner join               | ✅     |
| SJ2   | Join      |          | Left join                | ✅     |
| SJ3   | Join      |          | Left outer join          |        |
| SJ4   | Join      |          | Inner join ON            |        |
| SJ5   | Join      |          | Left join ON             |        |
| SJ6   | Join      |          | Left outer join ON       |        |
| SQ1   | Queryable |          | Queryable                | ✅     |
| SQ2   | Queryable |          | QueryableByName          |        |
| SQ3   | Queryable |          | Tuple of Queryable       | ✅     |
| SQ4   | Queryable |          | Tuple of QueryableByName |        |

### Insert

TODO

### Update

TODO

### Delete

TODO

### SQLite specials

#### Upsert

TODO

#### Returning

TODO

## How to run the tests

- `make test-watch` — spawn the worker and re-run tests when it is ready or hot-reloaded
- `make test-once` — spawn the worker, run tests once, then stop wrangler
- Or: `make test-worker-spawn` in one terminal, then `make test` in another

## How to add a new test

1. Add a new rust function like
   ```rust
   pub async fn test_users(env: &Env) {
    assert!(something_that_should_be_true);
   }
   ```
2. Add the function to the `tests!` macro in `src/lib.rs`
   ```rust
   tests!(test_users);
   ```
