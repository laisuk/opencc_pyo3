"""PEP 517 build backend with PDFium native preparation.

Delegates package building to maturin while ensuring the correct
PDFium native library is staged before wheel creation.

For ordinary pip installations, the host platform is detected
by the existing synchronization script.

For cross-compilation, set OPENCC_PDFIUM_TARGET explicitly.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path
from typing import Optional

from maturin import (
    build_sdist as _build_sdist,
    build_wheel as _build_wheel,
    get_requires_for_build_wheel,
    prepare_metadata_for_build_wheel as _prepare_metadata,
)

ROOT = Path(__file__).parent
TARGET_ENV = "OPENCC_PDFIUM_TARGET"


def _sync_pdfium() -> None:
    """Stage the target platform's PDFium native library.

    Uses the existing synchronization script as the single source
    of truth for platform detection and file copying.

    OPENCC_PDFIUM_TARGET overrides automatic platform detection,
    primarily for cross-compilation in CI.
    """
    target = os.environ.get(TARGET_ENV, "").strip()

    command = [
        sys.executable,
        str(ROOT / "tools" / "sync_pdfium_for_wheel.py"),
        "--local",
    ]

    if target:
        command.extend(["--target", target])

    subprocess.run(command, cwd=ROOT, check=True)


def build_wheel(
        wheel_directory: str,
        config_settings: Optional[dict] = None,
        metadata_directory: Optional[str] = None,
) -> str:
    """Synchronize PDFium natives and build a wheel with maturin."""
    _sync_pdfium()

    return _build_wheel(
        wheel_directory,
        config_settings=config_settings,
        metadata_directory=metadata_directory,
    )


def prepare_metadata_for_build_wheel(
        metadata_directory: str,
        config_settings: Optional[dict] = None,
) -> str:
    """Delegate wheel metadata generation to maturin."""
    return _prepare_metadata(metadata_directory, config_settings)


def build_sdist(
        sdist_directory: str,
        config_settings: Optional[dict] = None,
) -> str:
    """Build a source distribution using maturin."""
    return _build_sdist(sdist_directory, config_settings=config_settings)
