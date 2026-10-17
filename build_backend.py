
"""PEP 517 build backend with PDFium native preparation."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from maturin import (
    build_sdist as _build_sdist,
    build_wheel as _build_wheel,
    get_requires_for_build_wheel,
    prepare_metadata_for_build_wheel as _prepare_metadata,
)

ROOT = Path(__file__).parent


def _sync_pdfium() -> None:
    subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools" / "sync_pdfium_for_wheel.py"),
            "--local",
        ],
        cwd=ROOT,
        check=True,
    )


def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    _sync_pdfium()
    return _build_wheel(
        wheel_directory,
        config_settings=config_settings,
        metadata_directory=metadata_directory,
    )


def prepare_metadata_for_build_wheel(
        metadata_directory, config_settings=None
):
    return _prepare_metadata(metadata_directory, config_settings)
