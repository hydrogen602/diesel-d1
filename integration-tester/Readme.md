# How to run the tests

- `make test-watch` — spawn the worker and re-run tests when it is ready or hot-reloaded
- Or: `make test-worker-spawn` in one terminal, then `make test` in another

# How to add a new test

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
