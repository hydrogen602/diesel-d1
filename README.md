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
| S2    | Select    |          | Specific columns         | ✅     |
| S3    | Select    |          | Tuple of columns         | ✅     |
| SW1   | Where     | null     | `is null`                | ✅     |
| SW2   | Where     | null     | `is not null`            | ✅     |
| SW3   | Where     | int      | `<` int                  | ✅     |
| SW4   | Where     | int      | `>` int                  | ✅     |
| SW5   | Where     | int      | `<=` int                 | ✅     |
| SW6   | Where     | int      | `>=` int                 | ✅     |
| SW7   | Where     | int      | `=` int                  | ✅     |
| SW8   | Where     | int      | `<>` int                 | ✅     |
| SW9   | Where     | int      | `in` int[]               | ✅     |
| SW10  | Where     | int      | `not in` int[]           | ✅     |
| SW11  | Where     | int      | `between` int            | ✅     |
| SW12  | Where     | int      | `not between` int        | ✅     |
| SW13  | Where     | string   | `<` string               | ✅     |
| SW14  | Where     | string   | `>` string               | ✅     |
| SW15  | Where     | string   | `<=` string              | ✅     |
| SW16  | Where     | string   | `>=` string              | ✅     |
| SW17  | Where     | string   | `=` string               | ✅     |
| SW18  | Where     | string   | `<>` string              | ✅     |
| SW19  | Where     | string   | `in` string[]            | ✅     |
| SW20  | Where     | string   | `not in` string[]        | ✅     |
| SW21  | Where     | string   | `like` string            | ✅     |
| SW22  | Where     | string   | `not like` string        | ✅     |
| SW23  | Where     | Compound | `and`                    | ✅     |
| SW24  | Where     | Compound | `or`                     | ✅     |
| SW25  | Where     | Compound | `not`                    | ✅     |
| SO1   | Order by  |          | Single column            | ✅     |
| SO2   | Order by  |          | Multiple columns         | ✅     |
| SO3   | Order by  |          | Direction                | ✅     |
| SL1   | Limit     |          | Single value             | ✅     |
| SOff1 | Offset    |          | Single value             | ✅     |
| SJ1   | Join      |          | Inner join               | ✅     |
| SJ2   | Join      |          | Left join                | ✅     |
| SJ3   | Join      |          | Left outer join          | ✅     |
| SJ4   | Join      |          | Inner join ON            | ✅     |
| SJ5   | Join      |          | Left join ON             | ✅     |
| SJ6   | Join      |          | Left outer join ON       | ✅     |
| SQ1   | Queryable |          | Queryable                | ✅     |
| SQ2   | Queryable |          | QueryableByName          | ✅     |
| SQ3   | Queryable |          | Tuple of Queryable       | ✅     |
| SQ4   | Queryable |          | Tuple of QueryableByName | ✅     |

### Insert

| ID    | Category    | Type     | Case                     | Status |
| ----- | ----------- | -------- | ------------------------ | ------ |
| I1    | Insert      |          | All columns              | ✅     |
| I2    | Insert      |          | Specific columns         | ✅     |
| I3    | Insert      |          | Tuple of columns         | ✅     |
| I4    | Insert      |          | Insertable               | ✅     |
| I5    | Insert      |          | Default values           | ✅     |
| IB1   | Batch       |          | Multiple rows            | ✅     |
| IS1   | From select |          | Insert from select       | ✅     |
| IV1   | Values      | null     | `NULL`                   | ✅     |
| IV2   | Values      | null     | `Some`                   | ✅     |
| IE1   | Execute     |          | Affected row count       | ✅     |

### Update

| ID    | Category  | Type     | Case                     | Status |
| ----- | --------- | -------- | ------------------------ | ------ |
| U1    | Update    |          | All rows                 | ✅     |
| U2    | Update    |          | Single column            | ✅     |
| U3    | Update    |          | Multiple columns         | ✅     |
| U4    | Update    |          | AsChangeset              | ✅     |
| U5    | Update    |          | Identifiable             | ✅     |
| UW1   | Where     | int      | `=` int                  | ✅     |
| UW2   | Where     | Compound | `and`                    | ✅     |
| UW3   | Where     | int      | `in` int[]               | ✅     |
| UW4   | Where     | string   | `=` string               | ✅     |
| US1   | Set       | null     | Set `NULL`               | ✅     |
| US2   | Set       | null     | Set `Some`               | ✅     |
| US3   | Set       |          | Expression               | ✅     |
| UE1   | Execute   |          | Affected row count       | ✅     |
| UE2   | Execute   |          | Zero rows                | ✅     |

### Delete

| ID    | Category  | Type     | Case                     | Status |
| ----- | --------- | -------- | ------------------------ | ------ |
| Del1  | Delete    |          | All rows                 | ✅     |
| Del2  | Delete    |          | Identifiable             | ✅     |
| DelW1 | Where     | int      | `=` int                  | ✅     |
| DelW2 | Where     | Compound | `and`                    | ✅     |
| DelW3 | Where     | int      | `in` int[]               | ✅     |
| DelW4 | Where     | string   | `=` string               | ✅     |
| DelE1 | Execute   |          | Affected row count       | ✅     |
| DelE2 | Execute   |          | Zero rows                | ✅     |

### SQLite extras

#### Upsert

TODO

#### Returning

TODO

### D1 extras

| ID   | Category | Type | Case                                                        | Status |
| ---- | -------- | ---- | ----------------------------------------------------------- | ------ |
| D1T1 | D1       |      | Transactions are rejected                                   |        |
| D1T2 | D1       |      | Integers larger than `Number.MAX_SAFE_INTEGER` are rejected |        |

Other features todo:

- [ ] Session support
- [ ] Bookmark support
- [ ] Batch support
- [ ] Benchmark tests (compare to using D1 directly)

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
