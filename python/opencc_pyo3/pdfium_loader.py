import ctypes
import sys
from pathlib import Path
import os


def _detect_platform_folder() -> str:
    is_64bit = sys.maxsize > 2**32

    if sys.platform.startswith("win32") or sys.platform.startswith("cygwin"):
        arch = "x64" if is_64bit else "x86"
        return f"win-{arch}"

    elif sys.platform.startswith("linux"):
        machine = os.uname().machine
        if "aarch64" in machine or "arm64" in machine:
            arch = "arm64"
        elif "64" in machine:
            arch = "x64"
        else:
            arch = "x86"
        return f"linux-{arch}"

    elif sys.platform.startswith("darwin"):
        arch = "arm64" if os.uname().machine == "arm64" else "x64"
        return f"macos-{arch}"

    else:
        raise RuntimeError(f"Unsupported platform: {sys.platform}")


def load_pdfium() -> ctypes.CDLL:
    """Load bundled PDFium with correct calling convention."""

    base = Path(__file__).resolve().parent / "pdfium"
    platform_folder = _detect_platform_folder()
    path_dir = base / platform_folder

    if sys.platform.startswith("win"):
        libname = "pdfium.dll"
        dll_cls = ctypes.CDLL  # ALWAYS C CALLING CONVENTION (cdecl)
    elif sys.platform.startswith("linux"):
        libname = "libpdfium.so"
        dll_cls = ctypes.CDLL
    else:
        libname = "libpdfium.dylib"
        dll_cls = ctypes.CDLL

    lib_path = path_dir / libname

    if not lib_path.exists():
        raise RuntimeError(
            f"PDFium native library missing: {lib_path}\n"
            f"Expected platform folder: {platform_folder}"
        )

    try:
        return dll_cls(str(lib_path))
    except Exception as exc:
        raise RuntimeError(f"Failed to load PDFium: {exc}\nPath: {lib_path}")
