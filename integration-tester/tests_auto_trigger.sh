#!/bin/bash

# Runs the tests automatically when the worker is ready
# and whenever the worker is updated

# We loose color here but the script command attempt makes the output recognition fail
make test-worker-spawn | tee /dev/tty | while read -r line; do
    if [[ "$line" == *"[wrangler:info] Ready on http://localhost:8787"* ]]; then
        echo "Running tests..."
        make test
    elif [[ "$line" == *"Local server updated and ready"* ]]; then
        echo "Running tests..."
        make test
    fi
done
