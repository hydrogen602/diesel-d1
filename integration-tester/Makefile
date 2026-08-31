# So run `make worker-spawn` in one terminal
# and then run `make worker-test` in another terminal to run the tests

worker-spawn: setup-d1
	MAKEFLAGS= wrangler dev

# This triggers a scheduled event - the scheduled event will run the tests
worker-test:
	curl "http://localhost:8787/cdn-cgi/handler/scheduled"

setup-d1:
	wrangler d1 execute diesel-d1-test --file samples.sql --local
