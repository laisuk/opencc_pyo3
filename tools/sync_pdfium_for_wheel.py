from __future__ import annotations

import os
import platform
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "pdfium"
DST = ROOT / "opencc_pyo3" / "pdfium"

# Decide what you support
KNOWN = {"linux-x64", "macos-arm64", "win-x64"}


def die(msg: str) -> None:
    print(f"[sync_pdfium] ERROR: {msg}", file=sys.stderr)
    sys.exit(1)


def detect_target() -> str:
    sys_name = platform.system().lower()
    mach = platform.machine().lower()

    if sys_name == "windows":
        return "win-x64"
    if sys_name == "linux":
        return "linux-x64"
    if sys_name == "darwin":
        # If you later add macos-x64, extend this mapping + KNOWN set
        return "macos-arm64" if mach in ("arm64", "aarch64") else "macos-x64"

    die(f"unsupported platform: {sys_name} {mach}")
    raise SystemExit  # unreachable


def parse_mode(argv: list[str]) -> str:
    """
    Modes:
      - default / CI: strict + destructive (requires CI=true)
      - --local     : non-destructive (no deletions), for local dev
    """
    mode = "ci"
    for a in argv[1:]:
        if a == "--local":
            mode = "local"
        elif a in ("-h", "--help"):
            print(
                "Usage: python tools/sync_pdfium_for_wheel.py [--local]\n"
                "\n"
                "  (default)   CI/strict mode: deletes non-target dirs under opencc_pyo3/pdfium\n"
                "              and requires CI=true.\n"
                "  --local     Local mode: does NOT delete anything; only refreshes the target dir.\n"
            )
            sys.exit(0)
        else:
            die(f"unknown argument: {a}")
    return mode


def main() -> None:
    mode = parse_mode(sys.argv)
    is_ci = os.getenv("CI") == "true"

    if mode == "ci" and not is_ci:
        die("refusing to run destructive CI mode outside CI. Use --local, or set CI=true to override.")

    target = detect_target()
    if target not in KNOWN:
        die(f"detected target '{target}' is not in KNOWN={sorted(KNOWN)}")

    src_dir = SRC / target
    if not src_dir.exists():
        die(f"missing source natives dir: {src_dir}")

    # Ensure dst exists
    DST.mkdir(parents=True, exist_ok=True)

    # (CI mode only) Remove any non-target platform dirs from destination
    if mode == "ci":
        for d in DST.iterdir():
            if d.is_dir() and d.name != target:
                # Hard fail if it's not a known platform folder (catch mistakes)
                if d.name not in KNOWN:
                    die(f"unexpected folder inside opencc_pyo3/pdfium: {d}")
                print(f"[sync_pdfium] removing non-target dir: {d}")
                shutil.rmtree(d)
    else:
        print("[sync_pdfium] --local: non-destructive mode (will NOT delete other platform dirs)")

    # Recreate target dir cleanly (safe in both modes; only touches the target)
    dst_target = DST / target
    if dst_target.exists():
        print(f"[sync_pdfium] clearing target dir: {dst_target}")
        shutil.rmtree(dst_target)
    dst_target.mkdir(parents=True, exist_ok=True)

    # Copy platform natives into package
    print(f"[sync_pdfium] copying {src_dir} -> {dst_target}")
    for p in src_dir.iterdir():
        if p.is_file():
            shutil.copy2(p, dst_target / p.name)

    # Sanity check: at least one native file expected (adjust if you ship more)
    files = [p.name for p in dst_target.iterdir() if p.is_file()]
    if not files:
        die(f"no files copied into {dst_target}")
    print(f"[sync_pdfium] ok, copied files: {files}")


if __name__ == "__main__":
    main()
