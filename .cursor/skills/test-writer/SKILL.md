---
name: test-writer
description: Writes diesel-d1 integration tests with fully asserted output and README case tags (`// Test: S1 - All columns`). Use when adding or extending tests in integration-tester, covering untested README IDs, or when the user asks to write integration tests.
---

# Test writer

Write integration tests for this crate in `integration-tester/`, matching `integration-tester/src/basic_tests.rs`.

## Rules

Test output has to be fully asserted.

The code where something relevant happens has to be tagged like `// Test: S1 - All columns` based on the ids in `README.md`.

## Case IDs

Source of truth: the Integration testing tables in `README.md` (ID, Category, Type, Case, Status).

- Use the **ID** and **Case** columns for tags.
- Prefer untested rows (empty Status). Re-use an ID on another query only when it actually exercises that case again.
- Do not invent IDs. If a new case is needed, add the row to `README.md` first, then tag with that ID.

## Tags

Place the comment immediately above the Diesel builder or load that implements the case:

```rust
// Test: S1 - All columns
let query = sample_schema::users::table.select(sample_schema::users::all_columns);
```

- Format: `// Test: {ID} - {Case}` — Case text must match the README Case column.
- Stack tags when one expression covers multiple cases (see `test_users` / the join in `test_posts`).
- Tag the **relevant** line: `select` for S*, `filter` for SW*, `inner_join` / `left_join` for SJ*, `.load` type / `Queryable` for SQ*, and so on.
- The Makefile extractor is `//\s*Test:\s*\w+\b`. The ID must be a single word (`S1`, `SW3`, `SOff1`).

After adding tags, from `integration-tester/` run:

```bash
make update-readme-test-refs
```

That marks matching README rows ✅. It does not un-mark removed tags.

## Full assertion

Every test must assert the complete result, not a subset:

1. `assert_eq!(rows.len(), N)` with the exact expected count.
2. For **every** row, `assert_eq!` **every** field (including `Option` / joined columns).
3. Expected values come from `integration-tester/test_setup.sql` (and any data you add there).
4. Do not stop at `assert!(rows.len() > 0)`, partial field checks, or debug prints.

## Workflow

1. Read `README.md` for the IDs to cover.
2. Read `integration-tester/test_setup.sql` and existing tests in `integration-tester/src/`.
3. Extend fixtures/schema if the case needs data that is not there yet (`test_setup.sql` + the `diesel::table!` in the test module).
4. Add or extend an async `pub async fn test_*` in `integration-tester/src/` (keep using `basic_tests.rs` unless a new module is clearly needed).
5. Tag relevant code. Fully assert the output.
6. Register **new** functions in the `tests!` macro in `integration-tester/src/lib.rs`.
7. Run `make update-readme-test-refs` from `integration-tester/`.
8. Run tests from `integration-tester/` with `make test-watch` (or `make test` against an already spawned worker).

## Test function shape

```rust
pub async fn test_example(env: &Env) {
    let mut d1 = D1Connection::new(&env, D1_NAME).unwrap();

    // Test: SW4 - `>` int
    let query = sample_schema::users::table
        .select(sample_schema::users::all_columns)
        .filter(sample_schema::users::id.gt(2));

    let rows: Vec<User> = query.load(&mut d1).await.unwrap();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, 3);
    assert_eq!(rows[0].name, "Jim Beam");
    assert_eq!(rows[0].created_at, "2021-01-03");
    assert_eq!(rows[1].id, 4);
    assert_eq!(rows[1].name, "Jane Doe");
    assert_eq!(rows[1].created_at, "2021-01-04");
}
```

Register new names:

```rust
tests!(test_users, test_posts, test_example);
```

Reuse the existing `sample_schema`, `User`, and `Post` types when they fit. Add `Queryable` structs when a case needs a different shape.

## Fixtures

Default data (`test_setup.sql`):

| users.id | name       | created_at |
| -------- | ---------- | ---------- |
| 1        | John Doe   | 2021-01-01 |
| 2        | Jane Smith | 2021-01-02 |
| 3        | Jim Beam   | 2021-01-03 |
| 4        | Jane Doe   | 2021-01-04 |

| posts.id | title        | body                      | user_id |
| -------- | ------------ | ------------------------- | ------- |
| 1        | Hello World  | This is a test post       | 1       |
| 2        | Another Post | This is another test post | 1       |
| 3        | Post #3      | Lots of words             | 2       |
| 4        | Post #4      | Even more words           | 2       |
| 5        | Post #5      | Imagine a post here       | 2       |

Each HTTP test run re-executes this SQL (`setup_d1` in `lib.rs`), so tests start from this snapshot. Do not assume leftover writes from other tests.

## Do not

- Leave Status ✅ in README without a matching `// Test:` tag in `src/`.
- Tag a query that does not actually exercise that case.
- Swallow errors (`unwrap` in tests is fine; do not `let _ =` or ignore `Result` or not actually assert the desired condition to be true).
