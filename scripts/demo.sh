#!/usr/bin/env bash
# Build the interpreter, then run each example Lumen program and show its output.
set -euo pipefail

cargo build --release
BIN=./target/release/lumen

for example in fib closures fizzbuzz; do
    echo "== examples/${example}.lum =="
    "$BIN" run "examples/${example}.lum"
    echo
done
