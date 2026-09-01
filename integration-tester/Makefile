# Run `make test-watch` to spawn the worker and re-run tests on each reload.
# Run `make test-once` to spawn the worker and run the tests, and then exit.
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

# Future improvement: dynamically capture the url from the worker and pass it to the tests
test-once:
	@$(MAKE) test-worker-spawn | tee /dev/tty | { \
		while read -r line; do \
			case "$$line" in \
				*"[wrangler:info] Ready on http://localhost:8787"*|*"Local server updated and ready"*) \
					$(MAKE) test; \
					status=$$?; \
					pkill -INT -f "[w]rangler dev --local" || true; \
					exit $$status; \
					;; \
			esac; \
		done; \
		exit 1; \
	}

# 130 is SIGINT (Ctrl+C), 143 is SIGTERM (kill) 
# - we ignore these as that is how we stop the worker
test-worker-spawn:
	@MAKEFLAGS= wrangler dev --local --show-interactive-dev-session true || { \
		s=$$?; \
		[ $$s -eq 130 ] || [ $$s -eq 143 ]; \
	}

# This only works if test-worker-spawn is already running
test: tests.gen.py
	@echo "Running tests..."
	@python3 "$<"

FORCE: 

# This is run a bunch of times whenever files change so I want to keep log lines to a minimum
tests.gen.py: FORCE
	@echo "Generating tests..."
	@RUSTFLAGS="-A warnings" cargo run -q --bin gen_py_tests --target host-tuple

test-ref-finder:
	find src -type f -name '*.rs' -exec egrep -o "//\s*Test:\s*\w+\b" \{\} \; | awk -F: '{ gsub(/[ \t]+/, "", $$2); print $$2 }' | sort -u

# This updates the test coverage in the README.md file
# Whenever something is tested, add // Test: <id> to the test function
# Then run `make update-readme-test-refs` to update the README.md file
update-readme-test-refs:
	cat ../README.md | tr -d "✅" |             \
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
