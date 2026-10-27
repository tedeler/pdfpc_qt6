#!/usr/bin/env bash
# Build an amd64/arm64 .deb from rust/target/release/pdfpc-qt6.
# Package and binary are both pdfpc-qt6 so they coexist with classic pdfpc
# (Ubuntu package pdf-presenter-console → /usr/bin/pdfpc).
# Usage: packaging/build-deb.sh [output-dir]
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="${1:-"$ROOT/dist"}"
VERSION="$(tr -d '[:space:]' < "$ROOT/VERSION")"
ARCH="$(dpkg --print-architecture)"
PKG_NAME="pdfpc-qt6"
BIN_NAME="pdfpc-qt6"
BIN="$ROOT/rust/target/release/$BIN_NAME"

if [[ ! -x "$BIN" ]]; then
  echo "error: missing release binary at $BIN (run cargo build --release first)" >&2
  exit 1
fi

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

mkdir -p "$STAGE/usr/bin" "$STAGE/usr/share/doc/$PKG_NAME" "$STAGE/DEBIAN"
install -m 755 "$BIN" "$STAGE/usr/bin/$BIN_NAME"
strip --strip-unneeded "$STAGE/usr/bin/$BIN_NAME" 2>/dev/null || true

DESKTOP="$ROOT/packaging/pdfpc-qt6.desktop"
ICON_ROOT="$ROOT/packaging/icons/hicolor"
ICON_SVG="$ICON_ROOT/scalable/apps/pdfpc-qt6.svg"
if [[ ! -f "$DESKTOP" || ! -f "$ICON_SVG" ]]; then
  echo "error: missing $DESKTOP or $ICON_SVG" >&2
  exit 1
fi
install -D -m 644 "$DESKTOP" "$STAGE/usr/share/applications/${PKG_NAME}.desktop"
while IFS= read -r -d '' icon; do
  rel="${icon#"$ICON_ROOT"/}"
  install -D -m 644 "$icon" "$STAGE/usr/share/icons/hicolor/$rel"
done < <(find "$ICON_ROOT" -type f -print0)

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
  dpkg-shlibdeps -TDEBIAN/substvars -e "usr/bin/$BIN_NAME"
)
SHLIBS="$(sed -n 's/^shlibs:Depends=//p' "$STAGE/DEBIAN/substvars")"
# Qt PDF is loaded via QML plugins (not in NEEDED); keep those packages explicit.
RUNTIME_DEPS="libqt6pdf6, libqt6pdfquick6, qml6-module-qtquick, qml6-module-qtquick-controls, qml6-module-qtquick-layouts, qml6-module-qtquick-window, qml6-module-qtquick-pdf, qml6-module-qtqml, qml6-module-qtqml-workerscript, qml6-module-qtquick-dialogs, hicolor-icon-theme"
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
Homepage: https://github.com/tedeler/pdfpc_qt6
Description: Slim dual-monitor PDF presenter (Qt 6)
 pdfpc-qt6 is a slim dual-monitor PDF presenter for Linux (Qt 6 rewrite).
 Command: ${BIN_NAME}. Coexists with classic pdfpc (pdf-presenter-console).
 Built from git ${GIT_ID}.
EOF

rm -rf "$STAGE/debian" "$STAGE/DEBIAN/substvars"
mkdir -p "$OUT_DIR"
DEB_NAME="${PKG_NAME}_${VERSION}_${ARCH}.deb"
fakeroot dpkg-deb --build "$STAGE" "$OUT_DIR/$DEB_NAME"
echo "Wrote $OUT_DIR/$DEB_NAME"
