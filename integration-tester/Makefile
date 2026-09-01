# Run `make test-watch` to spawn the worker and re-run tests on each reload.
# Or run `make test-worker-spawn` and `make test` in separate terminals.

# We lose color here; `script` preserves it but breaks the output matching.
test-watch:
	@$(MAKE) test-worker-spawn | tee /dev/tty | while read -r line; do \
		case "$$line" in \
			*"[wrangler:info] Ready on http://localhost:8787"*|*"Local server updated and ready"*) \
				$(MAKE) test; \
				;; \
		esac; \
	done

test-worker-spawn:
	MAKEFLAGS= wrangler dev --local --show-interactive-dev-session true

test: tests.gen.py
	@echo "Running tests..."
	@python3 "$<"

FORCE: 

# This is run a bunch of times whenever files change so I want to keep log lines to a minimum
tests.gen.py: FORCE
	@echo "Generating tests..."
	@RUSTFLAGS="-A warnings" cargo run -q --bin gen_py_tests --target host-tuple

test-ref-finder:
	find src -type f -name '*.rs' -exec egrep -o "//\s*Test:\s*\w+\b" \{\} \; | awk -F: '{ print $2 }'

# This updates the test coverage in the README.md file
# Whenever something is tested, add // Test: <id> to the test function
# Then run `make update-readme-test-refs` to update the README.md file
#
# Note: doesn't un-mark tests that are no longer tested
update-readme-test-refs:
	cat ../README.md |                          \
		awk -F'|' -v OFS='|' -v                   \
		ids='$(shell $(MAKE) -s test-ref-finder)' \
		'BEGIN {                                  \
			split(ids, arr, /[ \t\n]+/);            \
			for (i in arr) targets[arr[i]] = 1;     \
		}                                         \
		{                                         \
			id=$$2;                                 \
			gsub(/[ \t]+/, "", id);                 \
			if (id in targets) {                    \
				$$6=" ✅     ";                       \
				print;                                \
			} else {                                \
				print;                                \
			}                                       \
		}' > tmp.md && mv tmp.md ../README.md
