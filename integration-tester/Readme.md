# How to run the tests

1. Run `make test-worker-spawn` in one terminal to spawn the worker
2. Run `make test` in another terminal to run the tests against the worker

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
