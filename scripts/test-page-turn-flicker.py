#!/usr/bin/env python3
"""Detect flicker (blank/empty flash) while advancing slides in pdfpc-qt6.

Captures a high-FPS X11 recording during page turns. A flash is a frame where
the main-slide crop is almost empty (dark presenter chrome) between content
frames — the remount gap from tearing down PdfPageImage.
"""

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
BIN = ROOT / "rust/target/release/pdfpc-qt6"
PDF = ROOT / "demo" / "beamer-madrid.pdf"
OUT = Path(os.environ.get("PDFPC_TEST_OUT", "/tmp/pdfpc-flicker-test"))
DISPLAY = os.environ.get("DISPLAY", ":99")
FPS = int(os.environ.get("PDFPC_FLICKER_FPS", "60"))
TURNS = int(os.environ.get("PDFPC_FLICKER_TURNS", "8"))

# Empty flash: remount leaves dark chrome where the slide was.
EMPTY_DARK_MIN = 0.70
CONTENT_DARK_MAX = 0.55


def run(cmd, check: bool = False, **kw):
    return subprocess.run(cmd, check=check, text=True, **kw)


def kill_pdfpc():
    run(["pkill", "-9", "-f", "pdfpc-qt6"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(0.3)


def write_recent(entries: list[dict]):
    cfg = Path(os.environ["XDG_CONFIG_HOME"]) / "pdfpc-qt6"
    cfg.mkdir(parents=True, exist_ok=True)
    (cfg / "recent.json").write_text(json.dumps({"files": entries}))


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


def focus_presenter():
    r = run(["xdotool", "search", "--name", "pdfpc-qt6 — presenter"], capture_output=True)
    wids = [w for w in (r.stdout or "").split() if w]
    if not wids:
        return None
    run(["xdotool", "windowfocus", wids[0]])
    return wids[0]


def frame_metrics(path: Path) -> dict:
    im = Image.open(path).convert("L")
    w, h = im.size
    crop = im.crop((int(w * 0.05), int(h * 0.08), int(w * 0.55), int(h * 0.72)))
    crop = crop.resize((120, 80))
    pixels = list(crop.get_flattened_data())
    n = len(pixels)
    mean = sum(pixels) / n
    std = (sum((p - mean) ** 2 for p in pixels) / n) ** 0.5
    dark = sum(1 for p in pixels if p < 45) / n
    bright = sum(1 for p in pixels if p > 240) / n
    return {
        "path": path.name,
        "mean": mean,
        "std": std,
        "dark": dark,
        "bright": bright,
        "empty": dark >= EMPTY_DARK_MIN,
        "content": dark <= CONTENT_DARK_MAX and std >= 25.0,
    }


def analyze_frames(frame_dir: Path) -> dict:
    frames = sorted(frame_dir.glob("frame_*.png"))
    stats = [frame_metrics(f) for f in frames]
    flashes: list[int] = []
    for i in range(1, len(stats) - 1):
        prev, cur, nxt = stats[i - 1], stats[i], stats[i + 1]
        if prev["content"] and cur["empty"] and (nxt["content"] or not nxt["empty"]):
            flashes.append(i)
        elif prev["content"] and cur["empty"]:
            flashes.append(i)
    # Deduplicate consecutive empty indices from one remount
    compact: list[int] = []
    for i in flashes:
        if not compact or i > compact[-1] + 2:
            compact.append(i)
    return {
        "frames": stats,
        "flash_indices": compact,
        "flash_count": len(compact),
        "empty_frame_count": sum(1 for s in stats if s["empty"]),
        "content_frame_count": sum(1 for s in stats if s["content"]),
    }


def capture_page_turns(turns: int, fps: int) -> Path:
    OUT.mkdir(parents=True, exist_ok=True)
    frames_dir = OUT / "frames"
    frames_dir.mkdir(exist_ok=True)
    for p in frames_dir.glob("*"):
        p.unlink()

    video = OUT / "turns.mp4"
    if video.exists():
        video.unlink()

    disp = DISPLAY.lstrip(":")
    ff = subprocess.Popen(
        [
            "ffmpeg", "-y",
            "-f", "x11grab", "-video_size", "1280x800", "-framerate", str(fps),
            "-i", f":{disp}.0",
            "-an",
            str(video),
        ],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    time.sleep(0.35)
    focus_presenter()
    for _ in range(turns):
        run(["xdotool", "key", "Right"])
        time.sleep(0.28)
    time.sleep(0.45)
    ff.send_signal(signal.SIGINT)
    try:
        ff.wait(timeout=5)
    except subprocess.TimeoutExpired:
        ff.kill()

    run(
        ["ffmpeg", "-y", "-i", str(video), "-vf", f"fps={fps}", str(frames_dir / "frame_%04d.png")],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return frames_dir


def main() -> int:
    if not BIN.exists():
        print(f"missing binary: {BIN}", file=sys.stderr)
        return 2
    if not PDF.exists():
        print(f"missing pdf: {PDF}", file=sys.stderr)
        return 2

    xdg = OUT / "xdg"
    xdg.mkdir(parents=True, exist_ok=True)
    os.environ["XDG_CONFIG_HOME"] = str(xdg)
    os.environ["DISPLAY"] = DISPLAY

    kill_pdfpc()
    write_recent([{"path": str(PDF.resolve()), "page": 0}])
    proc = start_pdfpc(PDF)
    time.sleep(2.5)
    if focus_presenter() is None:
        print("FAIL: presenter window not found", file=sys.stderr)
        kill_pdfpc()
        return 1

    time.sleep(0.4)
    frames_dir = capture_page_turns(turns=TURNS, fps=FPS)
    kill_pdfpc()
    try:
        proc.wait(timeout=2)
    except Exception:
        pass

    result = analyze_frames(frames_dir)
    # Drop bulky per-frame dump defaults; keep summary + flash frames
    slim = {
        "flash_count": result["flash_count"],
        "flash_indices": result["flash_indices"],
        "empty_frame_count": result["empty_frame_count"],
        "content_frame_count": result["content_frame_count"],
        "frame_count": len(result["frames"]),
        "flash_frames": [result["frames"][i] for i in result["flash_indices"]],
    }
    report = OUT / "report.json"
    report.write_text(json.dumps(slim, indent=2))
    (OUT / "report-full.json").write_text(json.dumps(result, indent=2))

    print(
        f"frames={slim['frame_count']} empty={slim['empty_frame_count']} "
        f"content={slim['content_frame_count']} flash_count={slim['flash_count']}"
    )
    for fr in slim["flash_frames"][:12]:
        print(
            f"  FLASH @{fr['path']} mean={fr['mean']:.1f} "
            f"dark={fr['dark']:.2f} std={fr['std']:.1f}"
        )
    print(f"report={report}")

    if slim["flash_count"] > 0:
        print("RESULT: FLICKER_DETECTED")
        return 1
    print("RESULT: NO_FLICKER")
    return 0


if __name__ == "__main__":
    sys.exit(main())
