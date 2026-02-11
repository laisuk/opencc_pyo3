import ctypes
import os
import sys
from pathlib import Path


def _detect_platform_folder() -> str:
    is_64bit = sys.maxsize > 2**32

    if sys.platform.startswith(("win32", "cygwin")):
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


def _module_dir() -> Path:
    """
    Return directory of opencc_pyo3 package.

    Supports:
    - normal pip install
    - virtualenv
    - mapped drives / subst
    - PyInstaller (onefile & onedir)
    """

    # PyInstaller onefile / onedir
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        return Path(sys._MEIPASS) / "opencc_pyo3"

    # Normal installed package
    return Path(os.path.abspath(__file__)).parent


def load_pdfium() -> ctypes.CDLL:
    """Load bundled PDFium."""
    base = _module_dir() / "pdfium"

    platform_folder = _detect_platform_folder()
    path_dir = base / platform_folder

    if sys.platform.startswith("win"):
        libname = "pdfium.dll"
        dll_cls = ctypes.CDLL
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
            f"Expected platform folder: {platform_folder}\n"
            f"module_dir={_module_dir()}"
        )

    try:
        return dll_cls(str(lib_path))
    except Exception as exc:
        raise RuntimeError(f"Failed to load PDFium: {exc}\nPath: {lib_path}")
