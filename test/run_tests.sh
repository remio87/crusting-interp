#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --quiet

BIN="target/debug/crusting-interp"
TEST_DIR="test"

for file in "$TEST_DIR"/*.lox; do
  echo "=== $file ==="
  "$BIN" "$file"
  echo
done
