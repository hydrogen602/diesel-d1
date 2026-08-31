# So run `make worker-spawn` in one terminal
# and then run `make worker-test` in another terminal to run the tests

test-worker-spawn: setup-d1
	MAKEFLAGS= wrangler dev --local

test: tests.gen.py
	python3 "$<"

# # This triggers a scheduled event - the scheduled event will run the tests
# worker-test:
# 	curl "http://localhost:8787/cdn-cgi/handler/scheduled"

setup-d1:
	wrangler d1 execute diesel-d1-test --file samples.sql --local

FORCE: 

tests.gen.py: FORCE
	cargo run --bin gen_py_tests --target host-tuple
