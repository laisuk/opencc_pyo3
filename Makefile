PYTHON ?= python3.13
WIN7_TARGET ?= x86_64-win7-windows-msvc
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
		maturin build --release --interpreter $(PYTHON) \
	"

develop:
	$(POWERSHELL) -NoProfile -Command "\
		$$env:RUSTFLAGS = $$null; \
		maturin develop --release --interpreter $(PYTHON) \
	"

# -------------------------
# Win7 (PowerShell, echoed, scoped)
# -------------------------

develop-win7:
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

build-win7:
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

# -------------------------
# Utilities
# -------------------------

clean:
	rm -rf build/ dist/ target/ *.egg-info

check:
	twine check --strict dist/*

publish:
	maturin upload --skip-existing dist/*
