# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and uses
the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

---

## [0.8.8] - 2026-02-12

### Added

- Added additional platform wheel artifacts in CI workflow:
  - Linux ARM64 (native runner)
  - Windows Win7 x86 / x64 (nightly build-std)
- Added `extract_pdf_text_pages_pdfium()` helper returning `List[str]` (one entry per page).

### Changed

- Deprecated legacy PDF extraction via the `pdf-extract` crate.
- PDFium is now the single source of truth for all PDF-related features.
- Legacy `pdf-extract` remains available when building from source with the `pdf-extract` feature enabled.
- Improved Pdfium native synchronization logic for wheel packaging.
- Improved transition error messages to clearly guide users to PDFium-based APIs.

### Fixed

- Fixed `pdfium_loader` path resolution to avoid `Path.resolve()` WinError 1 on:
  - Windows 7
  - Python 3.8 (especially 32-bit)
  - RAM disks / subst drives
  - PyInstaller and Nuitka environments
- Improved loader robustness for frozen application builds.

---

## [0.8.7] - 2026-01-26

### Changed

- **Lazy-load PDFium native backend**: PDFium is now imported and loaded only when the `pdf` subcommand is executed
  (and when `--engine auto|pdfium` is selected). This avoids eager native loading during normal library import
  and keeps `convert` / `office` workflows unaffected.

---

## [0.8.6] - 2026-01-26

### Fixed

- **macOS native loading path**: corrected the platform folder name from `osx-*` to `macos-*`, ensuring the PDFium
  native library is loaded from the correct directory on macOS systems.
- **Linux wheel packaging**: fixed an issue where `libpdfium.so` was not included in Linux wheels due to an imprecise
  packaging include pattern. The wheel build now explicitly includes all PDFium native files on Linux.

---

## [0.8.5] - 2026-01-25

> Note: `Version 0.8.5` has been **yanked** due to PDFium native loading issues. Please use `0.8.6` or later.

### Added

- Added separate prebuilt wheels for **macOS x86_64 (Intel)** and **macOS arm64 (Apple Silicon)** to improve
  installation compatibility on macOS platforms.

### Changed

- Updated `opencc-fmmseg` dependency to **v0.8.5**.

### Packaging

- Refined macOS wheel packaging to provide architecture-specific builds instead of a single arm64-only wheel.
- Continued bundling of platform-specific PDFium native libraries in all wheel distributions.

---

## [0.8.4] - 2026-01-12

### Added

- PDF support in the `opencc-pyo3` CLI (`pdf` subcommand), enabling direct PDF text extraction and conversion.

### Changed

- Updated `opencc-fmmseg` to v0.8.4.

### Packaging

- Bundled platform-specific PDFium native libraries into the generated wheel files.

---

## [0.8.3] - 2025-10-22

- Update `opencc-fmmseg` to v0.8.3

---

## [0.8.2] - 2025-10-05

- Update opencc-fmmseg to v0.8.2

---

## [0.8.1] - 2025-08-31

### Changed

- Update opencc-fmmseg to v0.8.1

### Planed

- Add artifacts for Windows 7 x86_64/x86_32

---

## [0.8.0] - 2025-08-19

### Changed

- Update opencc-fmmseg to v0.8.0

---

## [0.7.0] – 2025-07-14

### Added

- Add Office and Epub documents support in Chinese text conversion to CLI.

### Changed

- Code optimized.
- Dictionary optimization from Rust opencc-fmmseg repository.
- Optimized Chinese text code detection.
- Update opencc-fmmseg to v0.7.0
- Changed opencc-pyo3 CLI text conversion to subcommand convert.

---

## [0.6.2] – 2025-06-19

### Added

- Add set_config(), get_config() and supported_configs().
- Add opencc_py03 executable script.

### Changed

- Code optimized.
- Fixed type runtimes warnings in Python 3.8.
- Improved CLI argument parsing, help message formatting, and file encoding handling.

---

## [0.6.1] – 2025-06-12

### Added

- Initial release of `opencc-pyo3` on PyPI.
- Python bindings for Rust-powered OpenCC conversion using PyO3.
- Support for standard OpenCC conversion configs:
    - `s2t`, `s2tw`, `s2twp`, `s2hk`, `t2s`, `tw2s`, `tw2sp`, `hk2s`, `jp2t`, `t2jp`
- CLI tool: `python -m opencc_pyo3` with options for text conversion.
- Binary wheels for Linux, macOS, and Windows via `maturin`.
- UTF-8 encoding handling with fallback for BOM detection.

---
