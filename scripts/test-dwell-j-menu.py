#!/usr/bin/env python3
"""Regression tests: ≥5s dwell records history on leave; j-menu + digit jump."""

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
OUT = Path(os.environ.get("PDFPC_TEST_OUT", "/tmp/pdfpc-dwell-j-test"))
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


def start_pdfpc(pdf: Path, *, dwell_ms: int | None = 600) -> subprocess.Popen:
    env = os.environ.copy()
    env["DISPLAY"] = DISPLAY
    env["QT_QPA_PLATFORM"] = "xcb"
    if dwell_ms is None:
        env.pop("PDFPC_HISTORY_DWELL_MS", None)
    else:
        env["PDFPC_HISTORY_DWELL_MS"] = str(dwell_ms)
    return subprocess.Popen(
        [str(BIN), "--windowed", str(pdf)],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
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
        time.sleep(0.12)
    return False


def overlay_signal(path: Path) -> dict:
    from PIL import Image

    im = Image.open(path).convert("RGB")
    w, h = im.size
    crop = im.crop((w // 4, h // 4, 3 * w // 4, 3 * h // 4))
    dark = cyan = 0
    for px in crop.getdata():
        r, g, b = px
        if r < 50 and g < 50 and b < 50:
            dark += 1
        if b > 180 and g > 140 and r < 160:
            cyan += 1
    return {"dark": dark, "cyan": cyan}


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    os.environ.setdefault("XDG_CONFIG_HOME", str(OUT / "xdg-config"))
    os.environ.setdefault("XDG_RUNTIME_DIR", "/tmp/runtime-ci")
    Path(os.environ["XDG_RUNTIME_DIR"]).mkdir(parents=True, exist_ok=True)
    if not BIN.is_file():
        print(f"missing {BIN}", file=sys.stderr)
        return 2

    failures: list[str] = []
    dwell = 600  # ms — keep tests fast; production default remains 5000

    # A: dwell on page 0, arrow next → history records 0 (Ctrl+Left returns)
    print("=== A: sequential leave after dwell records history ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO, dwell_ms=dwell)
    time.sleep(2.4)
    if not focus_presenter():
        failures.append("A: no presenter")
    else:
        time.sleep(dwell / 1000.0 + 0.25)
        xdotool("key", "Right")
        if not wait_page(DEMO, 1):
            failures.append("A: Right failed")
        else:
            xdotool("key", "ctrl+Left")
            if not wait_page(DEMO, 0):
                failures.append(
                    f"A: Ctrl+Left expected page 0 after dwelled Next, got {read_recent_page(DEMO)}"
                )
            else:
                print("OK   dwelled Next records history")
    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    # B: short stop must NOT record
    print("=== B: short Zwischenstopp skipped ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO, dwell_ms=dwell)
    time.sleep(2.4)
    if not focus_presenter():
        failures.append("B: no presenter")
    else:
        time.sleep(dwell / 1000.0 + 0.25)
        xdotool("key", "g", "3", "Return")
        wait_page(DEMO, 2)
        time.sleep(0.1)  # << dwell
        xdotool("key", "g", "4", "Return")
        wait_page(DEMO, 3)
        xdotool("key", "ctrl+Left")
        if not wait_page(DEMO, 0):
            failures.append(
                f"B: expected skip short stop back to 0, got {read_recent_page(DEMO)}"
            )
        else:
            print("OK   short stop skipped")
    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    # C: j-menu visible + digit selects
    print("=== C: j-menu + digit ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO, dwell_ms=0)
    time.sleep(2.4)
    if not focus_presenter():
        failures.append("C: no presenter")
    else:
        xdotool("key", "Right")
        wait_page(DEMO, 1)
        time.sleep(0.2)
        xdotool("key", "j")
        time.sleep(0.45)
        shot = screenshot("C-j-menu.png")
        text = ocr(shot)
        (OUT / "C-j-menu.ocr.txt").write_text(text)
        sig = overlay_signal(shot)
        ok_ocr = "Jump" in text or "Slide" in text
        ok_px = sig["dark"] > 500 and sig["cyan"] > 15
        if not ok_ocr and not ok_px:
            failures.append(f"C: j-menu not visible ocr={text[:100]!r} sig={sig}")
        else:
            print("OK   j-menu visible")
        xdotool("key", "1")
        if not wait_page(DEMO, 0):
            failures.append(f"C: digit 1 expected page 0, got {read_recent_page(DEMO)}")
        else:
            print("OK   digit selects target")
    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    # D: j via empty-text fallback path — send key with xdotool keysym only
    print("=== D: j keysym opens menu ===")
    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO, dwell_ms=0)
    time.sleep(2.4)
    if not focus_presenter():
        failures.append("D: no presenter")
    else:
        xdotool("key", "Right")
        wait_page(DEMO, 1)
        # keysym j (same as letter); ensures Key_J path is exercised under xcb
        xdotool("key", "--clearmodifiers", "j")
        time.sleep(0.45)
        shot = screenshot("D-j-keysym.png")
        text = ocr(shot)
        if "Jump" not in text and "Slide" not in text and overlay_signal(shot)["cyan"] < 15:
            failures.append(f"D: j keysym did not open menu: {text[:100]!r}")
        else:
            print("OK   j keysym opens menu")
        xdotool("key", "Escape")
    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    if failures:
        print("FAILURES:")
        for f in failures:
            print(" ", f)
        return 1
    print("All dwell / j-menu checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
