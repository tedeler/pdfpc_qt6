#!/usr/bin/env python3
"""Old dwell-gate repro. The dwell gate is gone: only g, PDF links, and m push the stack.

Sequential keys do not. This script is not part of CI.
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "rust/target/release/pdfpc-qt6"
DEMO = ROOT / "demo" / "beamer-links.pdf"
OUT = Path(os.environ.get("PDFPC_TEST_OUT", "/tmp/pdfpc-dwell-j-repro"))
DISPLAY = os.environ.get("DISPLAY", ":1")


def run(cmd, check: bool = False, **kw):
    return subprocess.run(cmd, check=check, text=True, **kw)


def kill_pdfpc():
    run(["pkill", "-9", "-f", "pdfpc-qt6"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.3)


def write_recent(entries: list[dict]):
    cfg = Path(os.environ["XDG_CONFIG_HOME"]) / "pdfpc-qt6"
    cfg.mkdir(parents=True, exist_ok=True)
    (cfg / "recent.json").write_text(json.dumps({"files": entries}))


def read_recent_page(pdf: Path) -> int | None:
    p = Path(os.environ["XDG_CONFIG_HOME"]) / "pdfpc-qt6" / "recent.json"
    if not p.exists():
        return None
    data = json.loads(p.read_text())
    canon = str(pdf.resolve())
    for e in data.get("files", []):
        if e.get("path") == canon or Path(e.get("path", "")).name == pdf.name:
            return int(e.get("page", 0))
    return None


def start_pdfpc(pdf: Path, env_extra: dict | None = None) -> subprocess.Popen:
    env = os.environ.copy()
    env["DISPLAY"] = DISPLAY
    env["QT_QPA_PLATFORM"] = "xcb"
    # Real production dwell — do not set PDFPC_HISTORY_DWELL_MS.
    env.pop("PDFPC_HISTORY_DWELL_MS", None)
    if env_extra:
        env.update(env_extra)
    return subprocess.Popen(
        [str(BIN), "--windowed", str(pdf)],
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )


def xdotool(*args: str) -> str:
    r = run(["xdotool", *args], check=True, capture_output=True, env={**os.environ, "DISPLAY": DISPLAY})
    return (r.stdout or "").strip()


def focus_presenter() -> str | None:
    for _ in range(25):
        out = run(
            ["xdotool", "search", "--name", "presenter"],
            capture_output=True,
            env={**os.environ, "DISPLAY": DISPLAY},
        )
        wids = (out.stdout or "").split()
        if wids:
            wid = wids[0]
            xdotool("windowactivate", "--sync", wid)
            xdotool("windowfocus", "--sync", wid)
            return wid
        time.sleep(0.2)
    return None


def screenshot(name: str) -> Path:
    path = OUT / name
    run(["import", "-window", "root", str(path)], check=True, env={**os.environ, "DISPLAY": DISPLAY})
    return path


def ocr(path: Path) -> str:
    r = run(["tesseract", str(path), "stdout", "--psm", "6"], capture_output=True)
    return r.stdout or ""


def wait_page(pdf: Path, expected: int, timeout: float = 5.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        if read_recent_page(pdf) == expected:
            return True
        time.sleep(0.15)
    return False


def overlay_pixel_signal(path: Path) -> dict:
    """Detect dark modal + cyan key color from history overlay."""
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    # Center crop where the modal should sit
    crop = im.crop((w // 4, h // 4, 3 * w // 4, 3 * h // 4))
    dark = cyan = 0
    for px in crop.getdata():
        r, g, b = px
        if r < 50 and g < 50 and b < 50:
            dark += 1
        if b > 180 and g > 140 and r < 160:
            cyan += 1
    return {"dark": dark, "cyan": cyan, "pixels": crop.width * crop.height}


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    os.environ.setdefault("XDG_CONFIG_HOME", str(OUT / "xdg-config"))
    os.environ.setdefault("XDG_RUNTIME_DIR", "/tmp/runtime-ci")
    Path(os.environ["XDG_RUNTIME_DIR"]).mkdir(parents=True, exist_ok=True)

    if not BIN.is_file():
        print(f"missing {BIN}", file=sys.stderr)
        return 2

    failures: list[str] = []

    # --- 1: real 5s dwell then goto; short hop; Ctrl+Left should skip short stop ---
    print("=== 1: real 5s dwell gate ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO)
    time.sleep(2.5)
    if not focus_presenter():
        failures.append("1: no presenter")
    else:
        print("waiting 5.2s on page 0 for dwell...")
        time.sleep(5.2)
        xdotool("key", "g", "3", "Return")
        if not wait_page(DEMO, 2):
            failures.append(f"1: goto 3 failed page={read_recent_page(DEMO)}")
        else:
            time.sleep(0.2)  # short stop on page 2
            xdotool("key", "g", "4", "Return")
            if not wait_page(DEMO, 3):
                failures.append(f"1: goto 4 failed page={read_recent_page(DEMO)}")
            else:
                xdotool("key", "ctrl+Left")
                got = None
                # Prefer page 0 (skip short stop). If dwell broken (always record), land on 2.
                deadline = time.time() + 3
                while time.time() < deadline:
                    got = read_recent_page(DEMO)
                    if got in (0, 2):
                        break
                    time.sleep(0.1)
                if got == 0:
                    print("OK   real dwell: short stop skipped → page 0")
                elif got == 2:
                    failures.append("1: dwell gate FAILED — short stop was recorded (back→2)")
                else:
                    failures.append(f"1: unexpected page after Ctrl+Left: {got}")

    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    # --- 2: after a recorded jump, j must show overlay ---
    print("=== 2: j-menu after dwelled jump ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    # Use short dwell for setup speed, but exercise the same code path as production.
    proc = start_pdfpc(DEMO, env_extra={"PDFPC_HISTORY_DWELL_MS": "0"})
    time.sleep(2.5)
    if not focus_presenter():
        failures.append("2: no presenter")
    else:
        xdotool("key", "g", "3", "Return")
        wait_page(DEMO, 2)
        time.sleep(0.3)
        before = screenshot("2-before-j.png")
        xdotool("key", "j")
        time.sleep(0.5)
        after = screenshot("2-after-j.png")
        text = ocr(after)
        (OUT / "2-after-j.ocr.txt").write_text(text)
        sig = overlay_pixel_signal(after)
        print(f"OCR snippet: {text[:200]!r}")
        print(f"overlay pixels: {sig}")
        has_title = "Jump" in text or "history" in text.lower() or "Slide" in text
        has_pixels = sig["dark"] > 500 and sig["cyan"] > 20
        if not has_title and not has_pixels:
            failures.append(f"2: j-menu not visible (ocr/pixels) sig={sig}")
        else:
            print("OK   j-menu visible")
        # close
        xdotool("key", "Escape")
        time.sleep(0.2)

        # Also try Key_J via keysym in case text delivery differs
        xdotool("key", "j")
        time.sleep(0.4)
        xdotool("key", "1")
        if not wait_page(DEMO, 0, timeout=3):
            failures.append(f"2: picker 1 did not jump to page 0, got {read_recent_page(DEMO)}")
        else:
            print("OK   picker digit jumps")

    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)
        out = proc.stdout.read() if proc.stdout else ""
        (OUT / "pdfpc-stdout.txt").write_text(out or "")

    # --- 3: sequential next after 5s dwell — does history get an entry? (expectation probe) ---
    print("=== 3: probe sequential dwell → history (Ctrl+Left) ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO)  # real 5s
    time.sleep(2.5)
    if not focus_presenter():
        failures.append("3: no presenter")
    else:
        print("waiting 5.2s on page 0, then Right...")
        time.sleep(5.2)
        xdotool("key", "Right")
        if not wait_page(DEMO, 1):
            failures.append("3: Right failed")
        else:
            xdotool("key", "ctrl+Left")
            time.sleep(0.5)
            got = read_recent_page(DEMO)
            if got == 0:
                print("NOTE  sequential dwell IS in history (back→0)")
            else:
                print(f"NOTE  sequential dwell NOT in history (still page {got}) — likely user expectation gap")
            # Not a hard failure yet — documents current behavior.
            (OUT / "3-sequential-note.txt").write_text(f"after Ctrl+Left page={got}\n")

    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    if failures:
        print("FAILURES:")
        for f in failures:
            print(" ", f)
        return 1
    print("Repro script finished (see NOTE for sequential).")
    return 0


if __name__ == "__main__":
    try:
        from PIL import Image  # noqa: F401
    except ImportError:
        run(["sudo", "DEBIAN_FRONTEND=noninteractive", "apt-get", "install", "-y", "-qq", "python3-pil"], check=False)
    sys.exit(main())
