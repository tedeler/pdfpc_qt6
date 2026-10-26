#!/usr/bin/env bash
# Build an amd64/arm64 .deb from rust/target/release/pdfpc.
# Usage: packaging/build-deb.sh [output-dir]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-"$ROOT/dist"}"
VERSION="$(tr -d '[:space:]' < "$ROOT/VERSION")"
ARCH="$(dpkg --print-architecture)"
BIN="$ROOT/rust/target/release/pdfpc"

if [[ ! -x "$BIN" ]]; then
  echo "error: missing release binary at $BIN (run cargo build --release first)" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/usr/bin" "$STAGE/usr/share/doc/pdfpc" "$STAGE/DEBIAN"
install -m 755 "$BIN" "$STAGE/usr/bin/pdfpc"
strip --strip-unneeded "$STAGE/usr/bin/pdfpc" 2>/dev/null || true

cp "$ROOT/LICENSE" "$STAGE/usr/share/doc/pdfpc/copyright"
cat > "$STAGE/usr/share/doc/pdfpc/changelog.Debian" <<EOF
pdfpc (${VERSION}) unstable; urgency=medium

  * Upstream release ${VERSION}.

 -- pdfpc_qt6 contributors <pdfpc@localhost>  $(date -Ru)
EOF
gzip -9n "$STAGE/usr/share/doc/pdfpc/changelog.Debian"

# Shared-library deps from the linked binary; QML modules listed explicitly
# because they are loaded at runtime and do not appear in NEEDED.
mkdir -p "$STAGE/debian"
touch "$STAGE/debian/control"
(
  cd "$STAGE"
  dpkg-shlibdeps -TDEBIAN/substvars -e usr/bin/pdfpc
)
SHLIBS="$(sed -n 's/^shlibs:Depends=//p' "$STAGE/DEBIAN/substvars")"
# Qt PDF is loaded via QML plugins (not in NEEDED); keep those packages explicit.
RUNTIME_DEPS="libqt6pdf6, libqt6pdfquick6, qml6-module-qtquick, qml6-module-qtquick-controls, qml6-module-qtquick-layouts, qml6-module-qtquick-window, qml6-module-qtquick-pdf, qml6-module-qtqml, qml6-module-qtqml-workerscript"
DEPENDS="${SHLIBS}, ${RUNTIME_DEPS}"

GIT_ID="$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"

cat > "$STAGE/DEBIAN/control" <<EOF
Package: pdfpc
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: pdfpc_qt6 contributors <pdfpc@localhost>
Section: utils
Priority: optional
Depends: ${DEPENDS}
Homepage: https://github.com/tedeler/pdfpc_qt6
Description: Slim dual-monitor PDF presenter (Qt 6)
 pdfpc is a slim dual-monitor PDF presenter for Linux.
 Built from git ${GIT_ID}.
EOF

rm -rf "$STAGE/debian" "$STAGE/DEBIAN/substvars"
mkdir -p "$OUT_DIR"
DEB_NAME="pdfpc_${VERSION}_${ARCH}.deb"
fakeroot dpkg-deb --build "$STAGE" "$OUT_DIR/$DEB_NAME"
echo "Wrote $OUT_DIR/$DEB_NAME"
