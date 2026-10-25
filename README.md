# pdfpc (Rust / Qt 6)

Slim dual-monitor PDF presenter for Linux (KDE/Wayland). Hard-wired keys match the subset of classic pdfpc that this rewrite keeps.

## Build

System packages (Ubuntu/Kubuntu):

```bash
sudo apt install \
  build-essential g++ qmake6 \
  qt6-base-dev qt6-declarative-dev qt6-pdf-dev qt6-tools-dev \
  libqt6pdf6 libqt6pdfquick6 \
  qml6-module-qtquick qml6-module-qtquick-controls \
  qml6-module-qtquick-layouts qml6-module-qtquick-window \
  qml6-module-qtquick-pdf qml6-module-qtqml \
  qml6-module-qtqml-workerscript qt6-wayland
```

Rust **1.85+** (tested with 1.88). Then:

```bash
cd rust
export QMAKE=/usr/bin/qmake6 QT_VERSION_MAJOR=6 CXX=g++
cargo build --release
```

Binary: `rust/target/release/pdfpc`.

## Usage

```bash
pdfpc [--windowed|-w] presentation.pdf
```

- Two monitors: presenter on the primary screen, audience on the secondary (fullscreen).
- One monitor / CI: automatically falls back to `--windowed` with both windows on screen 0.
- `Shift+S` swaps presenter and audience screens at runtime.

## Keys

| Key | Action |
|---|---|
| ←/→, ↑/↓, Space, Enter, Page Up/Down | Previous / next (normal mode) |
| Left click / right click | Next / previous (normal mode) |
| Home / End | First / last page |
| `g`, digits, Enter | Go to page (1-based) |
| `b` | Blank audience (black) |
| `f` | Freeze audience on the current page + strokes |
| Escape | Clear blank / freeze / goto / pause / mode |
| `2` | Laser pointer |
| `3` | Pen (red stroke, fixed width) |
| `c` | Clear strokes on the current page |
| `1` | Normal mouse |
| `p` | Pause timer |
| Ctrl+T | Reset timer |
| Shift+S | Swap screens |

Timer starts when you leave page 1. Shown only on the presenter.

## Out of scope

`pdfpcrc`, sidecars, notes, overlays, overview, movies, links, transitions, REST, QR, D-Bus, eraser, pen colors, undo, persistent drawings, Windows.

## PDF rendering

Slides are drawn with Qt PDF (`PdfPageImage`) on an opaque white paper rectangle. That matches a normal viewer for PDFs that never paint a page background (common with Distiller datasheets): without the paper layer, dark presenter/audience chrome showed through and looked like a broken invert.

Check against Poppler/Evince using the Beamer decks in `demo/` (see `demo/README.md`).

## Dual-monitor acceptance (KWin)

1. Open `demo/pdfpc-demo.pdf` on Kubuntu with two displays.
2. Confirm presenter (current + next + page + timer) vs audience (current only).
3. Navigate, `Shift+S`, blank (`b`), freeze (`f`), goto (`g`), laser (`2`), pen (`3`/`c`).
4. If a window lands on the wrong screen under Wayland, report it — placement uses `Window.screen` + fullscreen; a KWin script can be added as a fallback.

## License

GPL-3.0-or-later.
