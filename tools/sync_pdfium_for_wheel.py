from __future__ import annotations

import os
import platform
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "pdfium"
DST = ROOT / "opencc_pyo3" / "pdfium"

# Decide what you support for packing into wheels
KNOWN = {
    "linux-x64",
    "linux-arm64",
    "macos-x64",
    "macos-arm64",
    "win-x64",
    "win-arm64",
}


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
        return "macos-arm64" if mach in ("arm64", "aarch64") else "macos-x64"

    die(f"unsupported platform: {sys_name} {mach}")
    raise SystemExit  # unreachable


def parse_args(argv: list[str]) -> tuple[str, str | None]:
    """
    Modes:
      - default / CI: strict + destructive (requires CI=true)
      - --local     : non-destructive (no deletions), for local dev

    Options:
      - --target <name> : override platform folder to copy (e.g. macos-x64)
    """
    mode = "ci"
    target_override: str | None = None

    i = 1
    while i < len(argv):
        a = argv[i]
        if a == "--local":
            mode = "local"
            i += 1
        elif a == "--target":
            if i + 1 >= len(argv):
                die("--target requires a value (e.g. --target macos-x64)")
            target_override = argv[i + 1]
            i += 2
        elif a in ("-h", "--help"):
            print(
                "Usage: python tools/sync_pdfium_for_wheel.py [--local] [--target <name>]\n"
                "\n"
                "  (default)     CI/strict mode: deletes non-target dirs under opencc_pyo3/pdfium\n"
                "                and requires CI=true.\n"
                "  --local       Local mode: does NOT delete anything; only refreshes the target dir.\n"
                "  --target NAME Override platform folder to copy (e.g. macos-x64, macos-arm64).\n"
            )
            sys.exit(0)
        else:
            die(f"unknown argument: {a}")

    return mode, target_override


def main() -> None:
    mode, target_override = parse_args(sys.argv)
    is_ci = os.getenv("CI") == "true"

    if mode == "ci" and not is_ci:
        die("refusing to run destructive CI mode outside CI. Use --local, or set CI=true to override.")

    target = target_override or detect_target()
    if target not in KNOWN:
        die(f"target '{target}' is not in KNOWN={sorted(KNOWN)}")

    src_dir = SRC / target
    if not src_dir.exists():
        die(f"missing source natives dir: {src_dir}")

    # Ensure dst exists
    DST.mkdir(parents=True, exist_ok=True)

    # (CI mode only) Remove any non-target platform dirs from destination
    if mode == "ci":
        for d in DST.iterdir():
            if d.is_dir() and d.name != target:
                if d.name not in KNOWN:
                    die(f"unexpected folder inside opencc_pyo3/pdfium: {d}")
                print(f"[sync_pdfium] removing non-target dir: {d}")
                shutil.rmtree(d)
    else:
        print("[sync_pdfium] --local: non-destructive mode (will NOT delete other platform dirs)")

    # Recreate target dir cleanly
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

    files = [p.name for p in dst_target.iterdir() if p.is_file()]
    if not files:
        die(f"no files copied into {dst_target}")

    print(f"[sync_pdfium] ok, copied files: {files}")


if __name__ == "__main__":
    main()
