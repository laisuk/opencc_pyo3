PYTHON ?= python3.13
WIN7_TARGET ?= x86_64-win7-windows-msvc
WIN7_X86_TARGET ?= i686-win7-windows-msvc
WIN7_TOOLCHAIN ?= nightly
WIN7_RUSTFLAGS ?= -C link-arg=/SUBSYSTEM:CONSOLE,6.01
WIN7_ZFLAGS ?= -Z build-std=std,panic_abort
POWERSHELL ?= pwsh

.PHONY: build develop build-win7 develop-win7 clean check publish

# -------------------------
# Normal builds (explicitly no Win7 flags)
# -------------------------

build:
	$(POWERSHELL) -NoProfile -Command "\
		$$env:RUSTFLAGS = $$null; \
		maturin build --release \
	"

develop:
	$(POWERSHELL) -NoProfile -Command "\
		$$env:RUSTFLAGS = $$null; \
		maturin develop --release \
	"

# -------------------------
# Win7 (PowerShell, echoed, scoped)
# -------------------------

develop-win7-x64:
	@echo PowerShell:
	@echo.
	@echo $$env:RUSTUP_TOOLCHAIN = "$(WIN7_TOOLCHAIN)"
	@echo $$env:RUSTFLAGS = "$(WIN7_RUSTFLAGS)"
	@echo.
	@echo maturin develop `
	@echo   --release `
	@echo   --target $(WIN7_TARGET) `
	@echo   $(WIN7_ZFLAGS)
	@echo.
	$(POWERSHELL) -NoProfile -Command "\
		$$env:RUSTUP_TOOLCHAIN = '$(WIN7_TOOLCHAIN)'; \
		$$env:RUSTFLAGS = '$(WIN7_RUSTFLAGS)'; \
		maturin develop ` \
		  --release ` \
		  --target $(WIN7_TARGET) ` \
		  $(WIN7_ZFLAGS) \
	"

build-win7-x64:
	@echo Correct maturin build command (Win11 host)
	@echo ------------------------------------------
	@echo Used to produce distributable wheel (.whl):
	@echo.
	@echo PowerShell:
	@echo.
	@echo $$env:RUSTUP_TOOLCHAIN = "$(WIN7_TOOLCHAIN)"
	@echo $$env:RUSTFLAGS = "$(WIN7_RUSTFLAGS)"
	@echo.
	@echo maturin build `
	@echo   --release `
	@echo   --target $(WIN7_TARGET) `
	@echo   $(WIN7_ZFLAGS)
	@echo.
	$(POWERSHELL) -NoProfile -Command "\
		$$env:RUSTUP_TOOLCHAIN = '$(WIN7_TOOLCHAIN)'; \
		$$env:RUSTFLAGS = '$(WIN7_RUSTFLAGS)'; \
		maturin build ` \
		  --release ` \
		  --target $(WIN7_TARGET) ` \
		  $(WIN7_ZFLAGS) \
	"

# ------------------------------------------------------------
# build-win7-x86
#
# Build a Windows 7–compatible 32-bit Python extension wheel
# on a modern Windows host (e.g. Windows 11).
#
# Target:
#   - Rust: i686-win7-windows-msvc
#   - Python: cp38-abi3 (requires Python 3.8 x86 at runtime)
#
# Notes:
#   - Uses nightly + build-std to support Win7
#   - Forces PE subsystem version 6.01 (Windows 7)
#   - Produces a distributable .whl via maturin
#   - Cleans pdfium native residues from other platforms before sync
# ------------------------------------------------------------
build-win7-x86:
	@echo ============================================================
	@echo Win7 x86 wheel build (host: Windows 10/11)
	@echo ============================================================
	@echo.
	@echo This command produces a distributable Python wheel (.whl)
	@echo targeting:
	@echo   - Windows 7 (x86, 32-bit)
	@echo   - Python 3.8 ABI (abi3)
	@echo.
	@echo Toolchain configuration:
	@echo   RUSTUP_TOOLCHAIN = $(WIN7_TOOLCHAIN)
	@echo   RUSTFLAGS        = $(WIN7_RUSTFLAGS)
	@echo.
	@echo Pdfium packaging:
	@echo   - Remove other-platform natives under opencc_pyo3/pdfium/*
	@echo   - Sync local Pdfium for target: win-x86
	@echo.
	@echo ------------------------------------------------------------
	@echo Executing build...
	@echo ------------------------------------------------------------
	$(POWERSHELL) -NoProfile -Command "& { \
		$$ErrorActionPreference = 'Stop'; \
		\
		Write-Host '==[1] Clean pdfium folders (keep win-x86 only)=='; \
		$$pdfiumRoot = Join-Path (Get-Location) 'opencc_pyo3/pdfium'; \
		Write-Host ('pdfiumRoot=' + $$pdfiumRoot); \
		if (Test-Path $$pdfiumRoot) { \
			Get-ChildItem -LiteralPath $$pdfiumRoot -Directory -ErrorAction SilentlyContinue | ForEach-Object { \
				if ($$_.Name -ne 'win-x86') { \
					Write-Host ('Removing: ' + $$_.FullName); \
					Remove-Item -LiteralPath $$_.FullName -Recurse -Force; \
				} \
			}; \
		} else { \
			New-Item -ItemType Directory -Path $$pdfiumRoot | Out-Null; \
		} \
		\
		Write-Host '==[2] Clear residues inside win-x86 folder=='; \
		$$winx86 = Join-Path $$pdfiumRoot 'win-x86'; \
		if (Test-Path $$winx86) { \
			Get-ChildItem -LiteralPath $$winx86 -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force; \
		} else { \
			New-Item -ItemType Directory -Path $$winx86 | Out-Null; \
		} \
		\
		Write-Host '==[3] Sync Pdfium natives: --local --target win-x86=='; \
		python tools/sync_pdfium_for_wheel.py --local --target win-x86; \
		Write-Host 'Contents of opencc_pyo3/pdfium after sync:'; \
		Get-ChildItem -LiteralPath $$pdfiumRoot -Recurse | ForEach-Object { Write-Host $$_.FullName }; \
		\
		Write-Host '==[4] Build wheel (maturin)=='; \
		$$env:RUSTUP_TOOLCHAIN = '$(WIN7_TOOLCHAIN)'; \
		$$env:RUSTFLAGS = '$(WIN7_RUSTFLAGS)'; \
		maturin build --release --target $(WIN7_X86_TARGET) $(WIN7_ZFLAGS); \
	}"

# -------------------------
# Utilities
# -------------------------

clean:
	rm -rf build/ dist/ target/ *.egg-info

check:
	twine check --strict dist/*

# publish:
# 	maturin upload --skip-existing dist/*
