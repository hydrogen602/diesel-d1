# Run `make test-watch` to spawn the worker and re-run tests on each reload.
# Or run `make test-worker-spawn` and `make test` in separate terminals.

test-worker-spawn: setup-d1
	MAKEFLAGS= wrangler dev --local --show-interactive-dev-session true

# We lose color here; `script` preserves it but breaks the output matching.
test-watch:
	$(MAKE) test-worker-spawn | tee /dev/tty | while read -r line; do \
		case "$$line" in \
			*"[wrangler:info] Ready on http://localhost:8787"*|*"Local server updated and ready"*) \
				echo "Running tests..."; \
				$(MAKE) test; \
				;; \
		esac; \
	done

test: tests.gen.py
	python3 "$<"

# # This triggers a scheduled event - the scheduled event will run the tests
# worker-test:
# 	curl "http://localhost:8787/cdn-cgi/handler/scheduled"

setup-d1:
	wrangler d1 execute diesel-d1-test --file samples.sql --local

FORCE: 

# THis is run a bunch of times whenever files change so I want to keep log lines to a minimum
tests.gen.py: FORCE
	@echo "Generating tests..."
	@RUSTFLAGS="-A warnings" cargo run -q --bin gen_py_tests --target host-tuple
