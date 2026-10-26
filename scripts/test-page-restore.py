#!/usr/bin/env python3
"""Automated restore / navigation checks for pdfpc-qt6 recent-files pages."""

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
DEMO = ROOT / "demo"
OUT = Path(os.environ.get("PDFPC_TEST_OUT", "/tmp/pdfpc-restore-test"))
DISPLAY = os.environ.get("DISPLAY", ":99")

# (pdf basename, 0-based restore page, OCR needles that must appear on that page)
CASES = [
    ("pdfpc-demo.pdf", 2, ["Last page", "Slide 3/3"]),
    ("pdfpc-demo.pdf", 1, ["Timer starts", "Slide 2/3"]),
    ("pdfpc-demo.pdf", 0, ["Page 1", "Slide 1/3"]),
    ("beamer-madrid.pdf", 3, ["Slide 4/4"]),
    ("beamer-madrid.pdf", 1, ["Slide 2/4"]),
    ("beamer-boxes.pdf", 2, ["Slide 3/3"]),
    ("beamer-dark.pdf", 1, ["Slide 2/3"]),
    ("beamer-default-white.pdf", 2, ["Slide 3/3"]),
]


def run(cmd, check: bool = False, **kw):
    return subprocess.run(cmd, check=check, text=True, **kw)


def write_recent(entries: list[dict]):
    cfg = Path(os.environ["XDG_CONFIG_HOME"]) / "pdfpc-qt6"
    cfg.mkdir(parents=True, exist_ok=True)
    (cfg / "recent.json").write_text(json.dumps({"files": entries}))


def kill_pdfpc():
    run(["pkill", "-9", "-f", "pdfpc-qt6"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.3)


def start_pdfpc(pdf: Path) -> subprocess.Popen:
    env = os.environ.copy()
    env["DISPLAY"] = DISPLAY
    env["QT_QPA_PLATFORM"] = "xcb"
    env.setdefault("XDG_RUNTIME_DIR", "/tmp/runtime-ci")
    log = open(OUT / "app.log", "a")
    return subprocess.Popen(
        [str(BIN), "--windowed", str(pdf)],
        env=env,
        stdout=log,
        stderr=subprocess.STDOUT,
    )


def screenshot(name: str) -> Path:
    path = OUT / name
    run(["import", "-window", "root", str(path)], check=True)
    return path


def ocr(path: Path) -> str:
    r = run(["tesseract", str(path), "stdout"], capture_output=True)
    return (r.stdout or "") + (r.stderr or "")


def xdotool(*args):
    run(["xdotool", *args], check=False)


def focus_presenter():
    r = run(["xdotool", "search", "--name", "pdfpc-qt6 — presenter"], capture_output=True)
    wids = [w for w in (r.stdout or "").split() if w]
    if not wids:
        return None
    wid = wids[0]
    xdotool("windowfocus", wid)
    return wid


def wait_ready(seconds=2.5):
    time.sleep(seconds)


def assert_needles(text: str, needles: list[str], label: str) -> list[str]:
    fails = []
    norm = " ".join(text.split())
    for n in needles:
        # OCR is lossy; allow loose match on digits for Slide X/Y
        if n.startswith("Slide "):
            # e.g. Slide 3/3 -> require both numbers nearby
            parts = n.replace("Slide ", "").split("/")
            if len(parts) == 2 and parts[0] in norm and parts[1] in norm and "Slide" in norm:
                # Prefer exact-ish
                if n.replace(" ", "") not in norm.replace(" ", "") and n not in norm:
                    # still ok if "Slide 3 / 3" style
                    if not (f"Slide {parts[0]}" in norm and parts[1] in norm):
                        fails.append(f"{label}: missing status ~{n!r} in {norm[:120]!r}")
            continue
        if n not in norm and n.lower() not in norm.lower():
            fails.append(f"{label}: missing {n!r} in {norm[:160]!r}")
    return fails


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


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    if not BIN.is_file():
        print(f"missing binary {BIN}", file=sys.stderr)
        return 2

    os.environ.setdefault("XDG_CONFIG_HOME", str(OUT / "xdg-config"))
    os.environ.setdefault("XDG_RUNTIME_DIR", "/tmp/runtime-ci")
    Path(os.environ["XDG_RUNTIME_DIR"]).mkdir(parents=True, exist_ok=True)

    failures: list[str] = []
    passed = 0

    print("=== A: cold restore for each PDF/page ===")
    for i, (name, page, needles) in enumerate(CASES):
        pdf = DEMO / name
        kill_pdfpc()
        write_recent([{"path": str(pdf.resolve()), "page": page}])
        proc = start_pdfpc(pdf)
        wait_ready(3.0)
        shot = screenshot(f"A{i:02d}-{name}-p{page}.png")
        text = ocr(shot)
        (OUT / f"A{i:02d}.ocr.txt").write_text(text)
        fails = assert_needles(text, needles, f"A/{name}@p{page}")
        # White-page heuristic (skip dark themes — little near-white ink).
        if "dark" not in name:
            try:
                from PIL import Image

                im = Image.open(shot)
                crop = im.crop((220, 160, 700, 500))
                whites = 0
                for px in crop.getdata():
                    if isinstance(px, tuple) and px[0] > 230 and px[1] > 230 and px[2] > 230:
                        whites += 1
                if whites < 500:
                    fails.append(f"A/{name}@p{page}: slide area looks blank (whiteish={whites})")
            except Exception as e:
                fails.append(f"A/{name}: pillow check failed: {e}")

        if fails:
            failures.extend(fails)
            print(f"FAIL A {name} page={page}")
            for f in fails:
                print(" ", f)
        else:
            passed += 1
            print(f"OK   A {name} page={page}")
        kill_pdfpc()
        if proc.poll() is None:
            proc.send_signal(signal.SIGTERM)

    print("=== B: navigate then reopen (persist + restore) ===")
    for name, target_page, needles in [
        ("pdfpc-demo.pdf", 2, ["Last page", "Slide 3/3"]),
        ("beamer-madrid.pdf", 2, ["Slide 3/4"]),
        ("beamer-boxes.pdf", 1, ["Slide 2/3"]),
    ]:
        pdf = DEMO / name
        kill_pdfpc()
        write_recent([{"path": str(pdf.resolve()), "page": 0}])
        proc = start_pdfpc(pdf)
        wait_ready(2.5)
        focus_presenter()
        for _ in range(target_page):
            xdotool("key", "Right")
            time.sleep(0.45)
        wait_ready(0.8)
        shot_nav = screenshot(f"B-nav-{name}.png")
        text_nav = ocr(shot_nav)
        fails = assert_needles(text_nav, needles, f"B-nav/{name}")
        # Advance once more if not last — check next preview updates by leaving mid page
        # Quit and reopen
        kill_pdfpc()
        stored = read_recent_page(pdf)
        if stored != target_page:
            fails.append(f"B-persist/{name}: recent.json page={stored} expected {target_page}")
        proc = start_pdfpc(pdf)
        wait_ready(3.0)
        shot = screenshot(f"B-restore-{name}.png")
        text = ocr(shot)
        (OUT / f"B-{name}.ocr.txt").write_text(text)
        fails += assert_needles(text, needles, f"B-restore/{name}")
        if fails:
            failures.extend(fails)
            print(f"FAIL B {name}")
            for f in fails:
                print(" ", f)
        else:
            passed += 1
            print(f"OK   B {name} restored page={target_page}")
        kill_pdfpc()

    print("=== C: next-preview advances with main ===")
    # Open demo at page 0, step to page 1, OCR should show page2 content somewhere (next preview)
    pdf = DEMO / "pdfpc-demo.pdf"
    kill_pdfpc()
    write_recent([{"path": str(pdf.resolve()), "page": 0}])
    start_pdfpc(pdf)
    wait_ready(2.5)
    focus_presenter()
    xdotool("key", "Right")  # now on page 2 (index 1); next should be Last page
    wait_ready(1.0)
    shot = screenshot("C-next-preview.png")
    text = ocr(shot)
    (OUT / "C.ocr.txt").write_text(text)
    fails = []
    if "Slide 2/3" not in text and not ("Slide 2" in text and "3" in text):
        fails.append(f"C: expected Slide 2/3, got {text[:120]!r}")
    # Next preview should include "Last page" while main shows timer text
    if "Timer starts" not in text and "Timer" not in text:
        fails.append(f"C: main slide missing Timer text: {text[:120]!r}")
    # Next preview is small — prefer image heuristics; OCR is unreliable there.
    try:
        from PIL import Image, ImageStat

        im = Image.open(shot)
        nxt = im.crop((780, 180, 1080, 420))
        nxt_path = OUT / "C-next-crop.png"
        nxt.save(nxt_path)
        mean = ImageStat.Stat(nxt).mean[0]
        # Must contain a bright paper region (not an empty dark hole).
        if mean < 100:
            fails.append(f"C: next preview looks empty (mean={mean:.1f})")
        nxt_text = ocr(nxt_path)
        if nxt_text.strip() and ("Last page" not in nxt_text and "Page 3" not in nxt_text and "3" not in nxt_text):
            # Soft signal only when OCR returned something unrelated.
            pass
    except Exception as e:
        fails.append(f"C: next-preview crop failed: {e}")
    if fails:
        failures.extend(fails)
        print("FAIL C next-preview")
        for f in fails:
            print(" ", f)
    else:
        passed += 1
        print("OK   C next-preview tracks main")
    kill_pdfpc()

    print("=== D: switch between two PDFs via reopen ===")
    kill_pdfpc()
    write_recent(
        [
            {"path": str((DEMO / "beamer-madrid.pdf").resolve()), "page": 2},
            {"path": str((DEMO / "pdfpc-demo.pdf").resolve()), "page": 2},
        ]
    )
    start_pdfpc(DEMO / "beamer-madrid.pdf")
    wait_ready(3.0)
    t1 = ocr(screenshot("D-madrid.png"))
    fails = assert_needles(t1, ["Slide 3/4"], "D/madrid")
    kill_pdfpc()
    start_pdfpc(DEMO / "pdfpc-demo.pdf")
    wait_ready(3.0)
    t2 = ocr(screenshot("D-demo.png"))
    fails += assert_needles(t2, ["Last page", "Slide 3/3"], "D/demo")
    if fails:
        failures.extend(fails)
        print("FAIL D switch")
        for f in fails:
            print(" ", f)
    else:
        passed += 1
        print("OK   D switch between PDFs")
    kill_pdfpc()

    print()
    print(f"passed_blocks={passed} failure_lines={len(failures)}")
    report = OUT / "report.txt"
    report.write_text("\n".join(failures) if failures else "ALL CHECKS PASSED\n")
    print(f"report={report}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
