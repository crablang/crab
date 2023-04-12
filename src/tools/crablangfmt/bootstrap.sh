#!/bin/bash

# Make sure you double check the diffs after running this script - with great
# power comes great responsibility.
# We deliberately avoid reformatting files with crablangfmt comment directives.

cargo build --release

target/release/crablangfmt src/lib.rs
target/release/crablangfmt src/bin/main.rs
target/release/crablangfmt src/cargo-fmt/main.rs

for filename in tests/target/*.rs; do
    if ! grep -q "crablangfmt-" "$filename"; then
        target/release/crablangfmt $filename
    fi
done
