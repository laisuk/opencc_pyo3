PYTHON=python3.13

.PHONY: build develop clean publish

build:
	maturin build --release --interpreter $(PYTHON)

develop:
	maturin develop --interpreter $(PYTHON)

clean:
	rm -rf build/ dist/ target/ *.egg-info

check:
	twine check --strict dist/*

publish:
	maturin upload --skip-existing dist/*
