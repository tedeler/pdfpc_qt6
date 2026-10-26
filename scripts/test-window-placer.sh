#!/usr/bin/env bash
# Command and geometry checks for the compositor placement backends.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust"

export PKG_CONFIG_PATH="${PKG_CONFIG_PATH:-/usr/lib/x86_64-linux-gnu/pkgconfig}"

OUT_DIR="${PDFPC_TEST_OUT:-/tmp/pdfpc-window-placer-test}"
mkdir -p "$OUT_DIR"
BIN="$OUT_DIR/test_window_placer"

# shellcheck disable=SC2046
g++ -std=c++17 \
  cpp/test_window_placer.cpp \
  -o "$BIN" \
  -Icpp \
  -fPIC \
  $(pkg-config --cflags --libs Qt6Core)

"$BIN"
echo "All window-placer checks passed."
