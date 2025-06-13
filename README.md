# opencc_pyo3
[![PyPI version](https://img.shields.io/pypi/v/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![Downloads](https://pepy.tech/badge/opencc-pyo3)](https://pepy.tech/project/opencc-pyo3)
[![Python Versions](https://img.shields.io/pypi/pyversions/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![License](https://img.shields.io/github/license/laisuk/opencc_pyo3)](https://github.com/laisuk/opencc_pyo3/blob/main/LICENSE)
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

## Benchmarks

```
Python 3.13.4 (tags/v3.13.4:8a526ec, Jun  3 2025, 17:46:04) [MSC v.1943 64 bit (AMD64)]
Platform: Windows-11-10.0.26100-SP0
Processor: Intel64 Family 6 Model 191 Stepping 2, GenuineIntel
```

### BENCHMARK RESULTS

| Method            | Config  | TextSize |     Mean |   StdDev |      Min |      Max |  Ops/sec | Chars/sec |
|:------------------|:--------|---------:|---------:|---------:|---------:|---------:|---------:|----------:|
| Convert_Small     | s2t     |       28 | 0.120 ms | 0.100 ms | 0.031 ms | 0.761 ms |     8351 |    233817 |
| Convert_Medium    | s2t     |      480 | 0.141 ms | 0.039 ms | 0.106 ms | 0.385 ms |     7089 |   3402675 |
| Convert_Large     | s2t     |     6400 | 0.629 ms | 0.046 ms | 0.581 ms | 1.106 ms |     1589 |  10169747 |
| Convert_XLarge    | s2t     |    46500 | 2.401 ms | 0.098 ms | 2.286 ms | 3.218 ms |      416 |  19366459 |
| Convert_Small     | s2tw    |       28 | 0.093 ms | 0.020 ms | 0.056 ms | 0.195 ms |    10722 |    300210 |
| Convert_Medium    | s2tw    |      480 | 0.260 ms | 0.028 ms | 0.223 ms | 0.436 ms |     3842 |   1844249 |
| Convert_Large     | s2tw    |     6400 | 0.962 ms | 0.034 ms | 0.907 ms | 1.138 ms |     1040 |   6655685 |
| Convert_XLarge    | s2tw    |    46500 | 3.318 ms | 0.113 ms | 3.147 ms | 3.894 ms |      301 |  14013366 |
| Convert_Small     | s2twp   |       28 | 0.146 ms | 0.026 ms | 0.097 ms | 0.271 ms |     6850 |    191812 |
| Convert_Medium    | s2twp   |      480 | 0.375 ms | 0.033 ms | 0.344 ms | 0.688 ms |     2670 |   1281552 |
| Convert_Large     | s2twp   |     6400 | 1.505 ms | 0.042 ms | 1.420 ms | 1.679 ms |      664 |   4252592 |
| Convert_XLarge    | s2twp   |    46500 | 4.974 ms | 0.087 ms | 4.824 ms | 5.494 ms |      201 |   9348373 |

### Throughput vs Size

![ThroughputVsSizeChart](https://github.com/laisuk/opencc_pyo3/blob/master/assets/throughput_vs_size.png)

## License

[MIT](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)

---

Powered by Rust, PyO3, and OpenCC.