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

Or for development (May require venv):

```sh
maturin develop -r
```

See [build.txt](https://github.com/laisuk/opencc_pyo3/blob/master/build.txt) for detailed build and install instructions.

## Usage

### Python

```python
from opencc_pyo3 import OpenCC

text = "“春眠不觉晓，处处闻啼鸟。”"
opencc = OpenCC("s2t")
converted = opencc.convert(text, punctuation=True)
print(converted)  # 「春眠不覺曉，處處聞啼鳥。」
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
    - 1 - Traditional, 2 - Simplified, 0 - others

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

---

| Method            | Config  |  TextSize |      Mean |    StdDev |       Min |       Max |   Ops/sec | Chars/sec |
|:------------------|:--------|----------:|----------:|----------:|----------:|----------:|----------:|----------:|
| Convert_Small     | s2t     |       100 |  0.118 ms |  0.097 ms |  0.049 ms |  0.811 ms |      8499 |    849910 |
| Convert_Medium    | s2t     |      1000 |  0.250 ms |  0.036 ms |  0.211 ms |  0.509 ms |      4004 |   4003531 |
| Convert_Large     | s2t     |     10000 |  0.845 ms |  0.060 ms |  0.775 ms |  1.420 ms |      1184 |  11835419 |
| Convert_XLarge    | s2t     |    100000 |  4.755 ms |  0.152 ms |  4.515 ms |  5.680 ms |       210 |  21030543 |
| Convert_Small     | s2tw    |       100 |  0.141 ms |  0.027 ms |  0.096 ms |  0.321 ms |      7111 |    711093 |
| Convert_Medium    | s2tw    |      1000 |  0.392 ms |  0.030 ms |  0.355 ms |  0.623 ms |      2552 |   2552127 |
| Convert_Large     | s2tw    |     10000 |  1.271 ms |  0.044 ms |  1.191 ms |  1.474 ms |       787 |   7869452 |
| Convert_XLarge    | s2tw    |    100000 |  6.317 ms |  0.139 ms |  6.004 ms |  7.250 ms |       158 |  15831322 |
| Convert_Small     | s2twp   |       100 |  0.204 ms |  0.028 ms |  0.132 ms |  0.380 ms |      4911 |    491118 |
| Convert_Medium    | s2twp   |      1000 |  0.598 ms |  0.039 ms |  0.527 ms |  0.747 ms |      1671 |   1671296 |
| Convert_Large     | s2twp   |     10000 |  1.942 ms |  0.061 ms |  1.823 ms |  2.223 ms |       515 |   5149357 |
| Convert_XLarge    | s2twp   |    100000 |  9.937 ms |  0.173 ms |  9.542 ms | 10.707 ms |       101 |  10063174 |

---


### Throughput vs Size

![ThroughputVsSizeChart](https://github.com/laisuk/opencc_pyo3/blob/master/assets/throughput_vs_size.png)

## License

[MIT](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)

---

Powered by Rust, PyO3, OpenCC and opencc-fmmseg.