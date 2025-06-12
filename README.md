# opencc_pyo3
[![PyPI version](https://img.shields.io/pypi/v/opencc_pyo3.svg)](https://pypi.org/project/opencc_pyo3/)
[![Build Status](https://github.com/laisuk/opencc_pyo3/actions/workflows/build.yml/badge.svg)](https://github.com/laisuk/opencc_pyo3/actions/workflows/build.yml)

`opencc_pyo3` is a Python extension module powered by [Rust](https://www.rust-lang.org/) and [PyO3](https://pyo3.rs/), providing fast and accurate conversion between different Chinese text variants using [OpenCC](https://github.com/BYVoid/OpenCC) algorithms.

## Features

- Convert between Simplified, Traditional, Hong Kong, Taiwan, and Japanese Kanji Chinese text.
- Fast and memory-efficient, leveraging Rust's performance.
- Easy-to-use Python API.
- Supports punctuation conversion and automatic text code detection.

## Supported Conversion Configurations

- `s2t`, `t2s`, `s2tw`, `tw2s`, `s2twp`, `tw2sp`, `s2hk`, `hk2s`, `t2tw`, `tw2t`, `t2twp`, `tw2tp`, `t2hk`, `hk2t`, `t2jp`, `jp2t`

## Installation

Build and install the Python wheel using [maturin](https://github.com/PyO3/maturin):

```sh
# In project root
maturin build --release
pip install ./target/wheels/opencc_pyo3-<version>-cp<pyver>-abi3-<platform>.whl
```

Or for development:

```sh
maturin develop -r
```

See [build.txt](https://github.com/laisuk/opencc_pyo3/blob/master/build.txt) for detailed build and install instructions.

## Usage

### Python

```python
from opencc_pyo3 import OpenCC

text = "春眠不觉晓，处处闻啼鸟。"
opencc = OpenCC("s2t")
converted = opencc.convert(text, punctuation=True)
print(converted)
```

### CLI

You can also use the CLI interface:

```sh
python -m opencc_pyo3 -i input.txt -o output.txt -c s2t --punct
```

## API

### Class: `OpenCC`

- `OpenCC(config: str = "s2t")`
    - `config`: Conversion configuration (see above).
- `convert(input: str, punctuation: bool = False) -> str`
    - Convert text with optional punctuation conversion.
- `zho_check(input: str) -> int`
    - Detects the code of the input text.

## Development

- Rust source: [src/lib.rs](https://github.com/laisuk/opencc_pyo3/blob/master/src/lib.rs)
- Python bindings: [opencc_pyo3/__init__.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__init__.py), [opencc_pyo3/opencc_pyo3.pyi](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/opencc_pyo3.pyi)
- CLI: [opencc_pyo3/__main__.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__main__.py)

## License

[MIT](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)

---

Powered by Rust, PyO3, and OpenCC.