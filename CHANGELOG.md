# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) and uses the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

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
