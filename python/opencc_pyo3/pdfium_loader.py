import ctypes
import os
import sys
from pathlib import Path


def _detect_platform_folder() -> str:
    is_64bit = sys.maxsize > 2 ** 32

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


def _module_file_dir() -> Path:
    return Path(os.path.abspath(__file__)).parent


def _candidate_module_dirs() -> list[Path]:
    """
    Return candidate package roots that may contain bundled runtime assets.

    Preference order preserves the current tested PyInstaller layout first,
    then falls back to module-relative and flatter frozen layouts.
    """
    candidates: list[Path] = []
    meipass = getattr(sys, "_MEIPASS", None)
    module_dir = _module_file_dir()

    if getattr(sys, "frozen", False) and meipass:
        meipass_path = Path(meipass)
        candidates.extend([
            meipass_path / "opencc_pyo3",
            module_dir,
            meipass_path,
        ])
    else:
        candidates.append(module_dir)

    deduped: list[Path] = []
    seen: set[str] = set()
    for candidate in candidates:
        key = os.path.normcase(os.path.normpath(str(candidate)))
        if key not in seen:
            seen.add(key)
            deduped.append(candidate)
    return deduped


def _module_dir() -> Path:
    """
    Return the physical directory of the installed ``opencc_pyo3`` package.

    This helper intentionally avoids ``Path.resolve()`` because it may raise
    ``WinError 1`` on certain Windows environments (e.g. subst drives,
    RAM disks, network-mapped drives).

    Supported environments:
    - Standard pip installation
    - Virtual environments (venv / pyenv)
    - Windows mapped / virtual drives
    - PyInstaller (onefile & onedir builds via ``sys._MEIPASS``)
    - Nuitka standalone / onefile builds

    In frozen applications, prefer the current tested package-preserving layout
    first, but fall back to module-relative or flatter extraction layouts when
    assets are bundled differently.
    """
    candidates = _candidate_module_dirs()
    for candidate in candidates:
        if (candidate / "pdfium").exists():
            return candidate
    return candidates[0]


def load_pdfium() -> ctypes.CDLL:
    """Load bundled PDFium."""
    # NOTE: do not use Path.resolve();
    # it can raise WinError 1 on some Windows drives (esp. CPython 3.8 x86)
    base = _module_dir() / "pdfium"

    platform_folder = _detect_platform_folder()
    path_dir = base / platform_folder

    if sys.platform.startswith("win"):
        libname = "pdfium.dll"
        dll_cls = ctypes.CDLL
        # Python 3.8+ Windows DLL search path fix
        try:
            os.add_dll_directory(str(path_dir))
        except (AttributeError, FileNotFoundError):
            # AttributeError -> Python < 3.8
            # FileNotFoundError -> directory missing (handled below)
            pass
    elif sys.platform.startswith("linux"):
        libname = "libpdfium.so"
        dll_cls = ctypes.CDLL
    else:
        libname = "libpdfium.dylib"
        dll_cls = ctypes.CDLL

    lib_path = path_dir / libname

    if not lib_path.exists():
        searched = "\n".join(
            f"- {candidate / 'pdfium' / platform_folder / libname}"
            for candidate in _candidate_module_dirs()
        )
        raise RuntimeError(
            f"PDFium native library missing: {lib_path}\n"
            f"Expected platform folder: {platform_folder}\n"
            f"module_dir={_module_dir()}\n"
            f"Searched:\n{searched}"
        )

    try:
        return dll_cls(str(lib_path))
    except Exception as exc:
        raise RuntimeError(f"Failed to load PDFium: {exc}\nPath: {lib_path}")
