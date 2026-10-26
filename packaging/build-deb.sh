#!/usr/bin/env bash
# Build an amd64/arm64 .deb from rust/target/release/pdfpc.
# Package name is pdfpc-qt6 so it can replace Ubuntu's pdf-presenter-console
# (classic pdfpc), which also ships /usr/bin/pdfpc.
# Usage: packaging/build-deb.sh [output-dir]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-"$ROOT/dist"}"
VERSION="$(tr -d '[:space:]' < "$ROOT/VERSION")"
ARCH="$(dpkg --print-architecture)"
PKG_NAME="pdfpc-qt6"
BIN="$ROOT/rust/target/release/pdfpc"

if [[ ! -x "$BIN" ]]; then
  echo "error: missing release binary at $BIN (run cargo build --release first)" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/usr/bin" "$STAGE/usr/share/doc/$PKG_NAME" "$STAGE/DEBIAN"
install -m 755 "$BIN" "$STAGE/usr/bin/pdfpc"
strip --strip-unneeded "$STAGE/usr/bin/pdfpc" 2>/dev/null || true

cp "$ROOT/LICENSE" "$STAGE/usr/share/doc/$PKG_NAME/copyright"
cat > "$STAGE/usr/share/doc/$PKG_NAME/changelog.Debian" <<EOF
${PKG_NAME} (${VERSION}) unstable; urgency=medium

  * Upstream release ${VERSION}.

 -- pdfpc_qt6 contributors <pdfpc@localhost>  $(date -Ru)
EOF
gzip -9n "$STAGE/usr/share/doc/$PKG_NAME/changelog.Debian"

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
Package: ${PKG_NAME}
Version: ${VERSION}
Architecture: ${ARCH}
Maintainer: pdfpc_qt6 contributors <pdfpc@localhost>
Section: utils
Priority: optional
Depends: ${DEPENDS}
Conflicts: pdf-presenter-console, pdfpc
Replaces: pdf-presenter-console, pdfpc
Provides: pdf-presenter-console
Homepage: https://github.com/tedeler/pdfpc_qt6
Description: Slim dual-monitor PDF presenter (Qt 6)
 pdfpc-qt6 is a slim dual-monitor PDF presenter for Linux (Qt 6 rewrite).
 It provides the pdfpc command and replaces the classic pdf-presenter-console
 package when both would install /usr/bin/pdfpc.
 Built from git ${GIT_ID}.
EOF

rm -rf "$STAGE/debian" "$STAGE/DEBIAN/substvars"
mkdir -p "$OUT_DIR"
DEB_NAME="${PKG_NAME}_${VERSION}_${ARCH}.deb"
fakeroot dpkg-deb --build "$STAGE" "$OUT_DIR/$DEB_NAME"
echo "Wrote $OUT_DIR/$DEB_NAME"
