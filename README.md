# pdfpc (Rust / Qt 6)

Slim dual-monitor PDF presenter for Linux (X11 and Wayland, including KDE, GNOME, Sway, and Hyprland). Hard-wired keys match the subset of classic pdfpc that this rewrite keeps.

Current version: see [`VERSION`](VERSION) (also shown under **Help → About pdfpc-qt6**, together with the build-time Git commit id).

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
  qml6-module-qtqml-workerscript qml6-module-qtquick-dialogs \
  qt6-wayland
```

Rust **1.85+** (tested with 1.88). Then:

```bash
cd rust
export QMAKE=/usr/bin/qmake6 QT_VERSION_MAJOR=6 CXX=g++
cargo build --release
```

Binary: `rust/target/release/pdfpc-qt6`.

### Debian package

After a release build:

```bash
packaging/build-deb.sh
# → dist/pdfpc-qt6_<version>_<arch>.deb
```

Package and command are both **pdfpc-qt6** (`/usr/bin/pdfpc-qt6`), so they coexist with classic `pdfpc` (`pdf-presenter-console`):

```bash
sudo apt install ./pdfpc-qt6_1.01_amd64.deb
# or: sudo dpkg -i pdfpc-qt6_1.01_amd64.deb
```

CI builds the `.deb` on **Ubuntu 26.04** (matching Kubuntu 26.04 library package names) and uploads it as an artifact on every push/PR. Pushing a version tag that matches `VERSION` (e.g. `v1.01`) additionally publishes a GitHub Release with the same package.

## Usage

```bash
pdfpc-qt6 [--windowed|-w] [presentation.pdf]
```

- Without a PDF argument, pdfpc-qt6 opens a recent-files dialog (same as `F3`). Pick a recent file or browse. For each file it remembers the last slide and restores it on open (`~/.config/pdfpc-qt6/recent.json`).
- Two monitors: the presenter view is always a normal window covering half the area of the primary screen (width and height × 1/√2), centered. The audience view is fullscreen on the secondary screen.
- `Shift+S` swaps the screens: the audience goes fullscreen on the other screen and the presenter is re-centered there with half its area.
- One monitor: the audience view is a normal window as well, since fullscreen would cover the presenter. When a second screen is connected later, pdfpc switches to the two-screen layout.
- `--windowed` keeps the audience view in a window too.
- Closing either window quits pdfpc.
- Watch the console for `pdfpc: screens=…` / `pdfpc: placing …`.

### Window placement

Wayland does not let a client position a normal window. pdfpc asks the compositor that is actually running, and prints the choice as `pdfpc: placer=…`:

| Session | `placer` | Presenter | Audience |
|---|---|---|---|
| X11 (not i3) | `direct` | Qt sets position and size | Fullscreen on its output |
| KDE Plasma Wayland | `kwin` | One-shot KWin script over D-Bus. This is the only backend that loads a compositor script. The script speaks Plasma 5 (`clientList`, `/<id>`) and Plasma 6 (`windowList`, `/Scripting/Script<id>`). | Fullscreen on its output |
| Sway, i3 | `sway`, `i3` | i3 IPC: floating window, centered, half the output area | Moved onto that output, then fullscreen |
| Hyprland | `hyprland` | Hyprland socket: floating window, centered | Moved onto that monitor, then fullscreen |
| Other Wayland (GNOME, COSMIC, niri, labwc, …) | `output` | Fullscreen on its output. These compositors do not place normal windows for a client. | Fullscreen on its output |
| macOS, Windows | `direct` | Same Qt geometry path as X11. Those builds do not link the Linux backends and are not covered by Linux CI. | Fullscreen on its screen |

`Shift+S` swaps the two outputs and places again. `--windowed` keeps the audience in a window; on `output` compositors that window stays where the compositor put it.

## Keys

| Key | Action |
|---|---|
| →, ↓, Space, Enter, Page Down / left click | Next slide (normal and laser mode) |
| ←, ↑, Backspace, Page Up / right click | Previous slide (normal and laser mode) |
| Home / End | First / last slide |
| `g`, digits, Enter | Go to slide (1-based) |
| `b` | Blank audience (black) |
| `f` | Freeze audience on the current slide + strokes |
| `1` | Normal mouse |
| `2` | Laser pointer (left-drag: free afterglow polyline; Shift+drag: straight line; fades 2s after release) |
| `+` / `-` | Larger / smaller laser pointer |
| `3` | Pen (red stroke, fixed width) |
| `c` | Clear strokes on the current slide |
| `s` | Start timer |
| `p` | Pause / resume timer |
| Ctrl+T | Reset timer |
| Shift+S | Swap presenter and audience screens |
| F3 | Recent files / open PDF (Escape continues the current PDF) |
| `h` | Show / hide the shortcut help (presenter only) |
| Escape | Close recent/help, cancel go-to, unblank, unfreeze, resume timer, normal mouse |

Keys work in both windows. While the help is open only `h`, Escape and `F3` act. While the recent-files dialog is open, Escape/`F3` closes it when a PDF is already loaded. Unlike classic pdfpc, `h` shows help instead of hiding the presentation. Quit by closing a window.

Timer starts when you leave page 1, or with `s` (needed again after Ctrl+T reset). Shown only on the presenter.

## Out of scope

`pdfpcrc`, sidecars, notes, overlays, overview, movies, links, transitions, REST, QR, D-Bus remote control, eraser, pen colors, undo, persistent drawings (ink), Windows.

## PDF rendering

Slides are drawn with Qt PDF (`PdfPageImage`) on an opaque white paper rectangle. That matches a normal viewer for PDFs that never paint a page background (common with Distiller datasheets): without the paper layer, dark presenter/audience chrome showed through and looked like a broken invert.

Check against Poppler/Evince using the Beamer decks in `demo/` (see `demo/README.md`).

## Dual-monitor acceptance

1. Open `demo/pdfpc-demo.pdf` with two displays.
2. Confirm presenter (current + next + page + timer) vs audience (current only), and that each is on the screen named in `pdfpc: placing … via <placer>`.
3. Navigate, `Shift+S`, blank (`b`), freeze (`f`), goto (`g`), laser (`2`), pen (`3`/`c`).
4. On `direct`, `kwin`, `sway`, `i3`, and `hyprland` the presenter is a centered window of half the output area. On `output` (GNOME and other Wayland compositors) both windows are fullscreen, one per screen.

## License

GPL-3.0-or-later.
