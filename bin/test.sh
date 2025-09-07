#!/usr/bin/env bash
set -euo pipefail
PRONTODB_BIN="${PRONTODB_BIN:-$(pwd)/target/debug/prontodb}"
export PRONTODB_BIN
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$ROOT_DIR/target/test-bin"
mkdir -p "$BUILD_DIR"
compile_run() {
  local src="$1"; local out="$BUILD_DIR/$(basename "${src%.rs}")"
  echo "==> rustc $src -> $out"
  rustc "$src" -o "$out"
  echo "==> run $(basename "$out")"
  "$out"
}
echo "Using PRONTODB_BIN=$PRONTODB_BIN"
[[ -f "$ROOT_DIR/test.rs" ]] && compile_run "$ROOT_DIR/test.rs" || echo "skip test.rs"
[[ -f "$ROOT_DIR/test-tdd.rs" ]] && compile_run "$ROOT_DIR/test-tdd.rs" || echo "skip test-tdd.rs"
echo "ALL TESTS COMPLETE"
