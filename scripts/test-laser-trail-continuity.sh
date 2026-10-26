#!/usr/bin/env bash
# Verify laser afterglow paints a continuous white core + red band.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT/rust"

export QMAKE="${QMAKE:-/usr/bin/qmake6}"
export QT_VERSION_MAJOR="${QT_VERSION_MAJOR:-6}"
export CXX="${CXX:-g++}"
export CC="${CC:-gcc}"
export RUSTFLAGS="${RUSTFLAGS:--C linker=g++}"
export PKG_CONFIG_PATH="${PKG_CONFIG_PATH:-/usr/lib/x86_64-linux-gnu/pkgconfig}"

echo "== Rust polyline unit tests =="
cargo test --bin pdfpc-qt6 laser_trail -- --nocapture

echo "== Qt render self-test (white interior + red border ribbon) =="
OUT_DIR="${PDFPC_TEST_OUT:-/tmp/pdfpc-laser-trail-test}"
mkdir -p "$OUT_DIR"
BIN="$OUT_DIR/test_laser_trail_geom"

# shellcheck disable=SC2046
g++ -std=c++17 \
  cpp/test_laser_trail_geom.cpp \
  -o "$BIN" \
  -Icpp \
  -fPIC \
  $(pkg-config --cflags --libs Qt6Gui)

(cd "$OUT_DIR" && "$BIN")
echo "All laser-trail continuity checks passed."
