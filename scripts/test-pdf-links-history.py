#!/usr/bin/env python3
"""Exercise PDF-link jumps and Ctrl+Left/Right history in pdfpc-qt6."""

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
OUT = Path(os.environ.get("PDFPC_TEST_OUT", "/tmp/pdfpc-links-test"))
DISPLAY = os.environ.get("DISPLAY", ":99")


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


def start_pdfpc(pdf: Path) -> subprocess.Popen:
    env = os.environ.copy()
    env["DISPLAY"] = DISPLAY
    env["QT_QPA_PLATFORM"] = "xcb"
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
    # Prefer the presenter title.
    for _ in range(20):
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
    run(
        ["import", "-window", "root", str(path)],
        check=True,
        env={**os.environ, "DISPLAY": DISPLAY},
    )
    return path


def ocr(path: Path) -> str:
    r = run(
        ["tesseract", str(path), "stdout", "--psm", "6"],
        capture_output=True,
        env={**os.environ, "DISPLAY": DISPLAY},
    )
    return (r.stdout or "") + (r.stderr or "")


def wait_page(pdf: Path, expected: int, timeout: float = 4.0) -> bool:
    deadline = time.time() + timeout
    while time.time() < deadline:
        page = read_recent_page(pdf)
        if page == expected:
            return True
        time.sleep(0.15)
    return False


def link_rects_page1() -> list[tuple[float, float, float, float]]:
    """Return link rectangles in PDF user space (bottom-left origin)."""
    # mutool show dumps annotations; parse Rect lines for page 1 links.
    r = run(["mutool", "show", str(DEMO), "grep", "Subtype/Link"], capture_output=True, check=True)
    rects = []
    for line in (r.stdout or "").splitlines():
        if "/Rect[" not in line or "/Subtype/Link" not in line:
            continue
        # Only page-1 content stream objects also carry Navigation destinations.
        if "Navigation" not in line and "Outline" not in line and "GoTo" not in line:
            continue
        start = line.index("/Rect[") + len("/Rect[")
        end = line.index("]", start)
        nums = [float(x) for x in line[start:end].split()]
        if len(nums) == 4:
            rects.append((nums[0], nums[1], nums[2], nums[3]))
    return rects


def click_pdf_point(wid: str, pdf_x: float, pdf_y_top: float, page_w: float, page_h: float):
    """Click a point given in PDF top-left coordinates inside the presenter slide."""
    geom = xdotool("getwindowgeometry", "--shell", wid)
    props = dict(line.split("=", 1) for line in geom.splitlines() if "=" in line)
    win_x, win_y = int(props["X"]), int(props["Y"])
    win_w, win_h = int(props["WIDTH"]), int(props["HEIGHT"])

    # Approximate layout from Presenter.qml: margins 16, topRow ~92% height,
    # current slide 72% of row width, paper letterboxed with PreserveAspectFit.
    margin = 16
    bottom_bar = max(40, round(win_h * 0.08))
    row_w = win_w - 2 * margin
    row_h = win_h - 2 * margin - bottom_bar - 12
    slide_w = row_w * 0.72
    slide_h = float(row_h)
    slide_x = win_x + margin
    slide_y = win_y + margin

    fit = min(slide_w / page_w, slide_h / page_h)
    paper_w = page_w * fit
    paper_h = page_h * fit
    paper_x = slide_x + (slide_w - paper_w) / 2
    paper_y = slide_y + (slide_h - paper_h) / 2

    cx = int(paper_x + (pdf_x / page_w) * paper_w)
    cy = int(paper_y + (pdf_y_top / page_h) * paper_h)
    xdotool("mousemove", "--sync", str(cx), str(cy))
    xdotool("click", "1")
    return cx, cy


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    if not BIN.is_file():
        print(f"missing binary {BIN}", file=sys.stderr)
        return 2
    if not DEMO.is_file():
        print(f"missing demo {DEMO}", file=sys.stderr)
        return 2

    os.environ.setdefault("XDG_CONFIG_HOME", str(OUT / "xdg-config"))
    os.environ.setdefault("XDG_RUNTIME_DIR", "/tmp/runtime-ci")
    Path(os.environ["XDG_RUNTIME_DIR"]).mkdir(parents=True, exist_ok=True)

    failures: list[str] = []
    page_w, page_h = 362.835, 272.126

    kill_pdfpc()
    write_recent([{"path": str(DEMO.resolve()), "page": 0}])
    proc = start_pdfpc(DEMO)
    time.sleep(2.5)
    wid = focus_presenter()
    if not wid:
        failures.append("presenter window not found")
        print("FAIL: no presenter window")
        kill_pdfpc()
        return 1

    # --- A: go-to jump pushes history; Ctrl+Left / Ctrl+Right ---
    print("=== A: goto + history back/forward ===")
    xdotool("key", "g")
    time.sleep(0.15)
    xdotool("key", "3")
    time.sleep(0.15)
    xdotool("key", "Return")
    if not wait_page(DEMO, 2):
        failures.append(f"A: after goto 3 expected page 2, got {read_recent_page(DEMO)}")
    else:
        print("OK   goto page 3")

    shot = screenshot("A-after-goto.png")
    text = ocr(shot)
    (OUT / "A-after-goto.ocr.txt").write_text(text)
    if "Beta" not in text and "beta" not in text.lower():
        # Soft check — OCR may miss; page index is authoritative.
        print("note: OCR did not clearly see Beta; page index OK" if read_recent_page(DEMO) == 2 else "")

    xdotool("key", "ctrl+Left")
    if not wait_page(DEMO, 0):
        failures.append(f"A: Ctrl+Left expected page 0, got {read_recent_page(DEMO)}")
    else:
        print("OK   Ctrl+Left back to TOC")

    xdotool("key", "ctrl+Right")
    if not wait_page(DEMO, 2):
        failures.append(f"A: Ctrl+Right expected page 2, got {read_recent_page(DEMO)}")
    else:
        print("OK   Ctrl+Right forward to Beta")

    # --- B: click TOC link (Alpha → page 1) ---
    print("=== B: click TOC PDF link ===")
    xdotool("key", "Home")  # sequential; does not clear jump history, but leaves us on TOC
    time.sleep(0.4)
    # Ensure we are on page 0 without relying on Home as a jump.
    if read_recent_page(DEMO) != 0:
        # Use history / goto to TOC
        xdotool("key", "g", "1", "Return")
        wait_page(DEMO, 0)

    rects = link_rects_page1()
    if len(rects) < 1:
        failures.append("B: no link rectangles found in beamer-links.pdf")
    else:
        # Prefer the first Navigation link (Alpha); PDF y is bottom-left.
        x0, y0, x1, y1 = rects[0]
        mid_x = (x0 + x1) / 2
        mid_y_bottom = (y0 + y1) / 2
        mid_y_top = page_h - mid_y_bottom
        cx, cy = click_pdf_point(wid, mid_x, mid_y_top, page_w, page_h)
        print(f"clicked link at screen ({cx},{cy}) pdf=({mid_x:.1f},{mid_y_top:.1f}) rects={len(rects)}")
        if not wait_page(DEMO, 1, timeout=5.0):
            # Try second/third TOC entries if first hit failed (nav chrome links).
            jumped = False
            for idx, (x0, y0, x1, y1) in enumerate(rects[1:6], start=1):
                focus_presenter()
                xdotool("key", "g", "1", "Return")
                wait_page(DEMO, 0)
                mid_x = (x0 + x1) / 2
                mid_y_top = page_h - (y0 + y1) / 2
                click_pdf_point(wid, mid_x, mid_y_top, page_w, page_h)
                if wait_page(DEMO, 1, timeout=2.0) or wait_page(DEMO, 2, timeout=0.5) or wait_page(DEMO, 3, timeout=0.5):
                    jumped = True
                    print(f"OK   TOC link click via rect[{idx}] → page {read_recent_page(DEMO)}")
                    break
            if not jumped and read_recent_page(DEMO) in (1, 2, 3):
                jumped = True
                print(f"OK   TOC link click → page {read_recent_page(DEMO)}")
            if not jumped:
                failures.append(
                    f"B: TOC click did not change page (still {read_recent_page(DEMO)}); rects={rects[:5]}"
                )
                screenshot("B-link-miss.png")
        else:
            print("OK   TOC link click → Alpha (page 1)")

        # History back should return to TOC.
        page_after = read_recent_page(DEMO)
        if page_after and page_after > 0:
            xdotool("key", "ctrl+Left")
            if not wait_page(DEMO, 0):
                failures.append(f"B: Ctrl+Left after link expected 0, got {read_recent_page(DEMO)}")
            else:
                print("OK   Ctrl+Left after link → TOC")

    kill_pdfpc()
    if proc.poll() is None:
        proc.send_signal(signal.SIGTERM)

    if failures:
        print("FAILURES:")
        for f in failures:
            print(" ", f)
        return 1
    print("All pdf-link / history checks passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
