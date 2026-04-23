# opencc_pyo3

[![PyPI version](https://img.shields.io/pypi/v/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![Downloads](https://pepy.tech/badge/opencc-pyo3)](https://pepy.tech/project/opencc-pyo3)
[![Python Versions](https://img.shields.io/pypi/pyversions/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![License](https://img.shields.io/github/license/laisuk/opencc_pyo3)](https://github.com/laisuk/opencc_pyo3/blob/main/LICENSE)
[![Build Status](https://github.com/laisuk/opencc_pyo3/actions/workflows/build.yml/badge.svg)](https://github.com/laisuk/opencc_pyo3/actions/workflows/build.yml)

`opencc_pyo3` is a Python extension module powered by [Rust](https://www.rust-lang.org/) and [PyO3](https://pyo3.rs/),
providing fast and accurate conversion between different Chinese text variants
using [OpenCC](https://github.com/BYVoid/OpenCC) algorithms.

## Features

- Convert between Simplified, Traditional, Hong Kong, Taiwan, and Japanese Kanji variants with OpenCC-compatible
  configurations.
- High-performance Rust + PyO3 backend for fast, memory-efficient Chinese text conversion in Python.
- Python API with `OpenCC`, `OpenccConfig`, config validation helpers, punctuation conversion, and Chinese text variant
  detection.
- Command-line interface for plain text conversion from files or standard input.
- Office and EPUB document conversion support for `.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`, and `.epub`.
- PDF text extraction helpers, including PDFium-based page-by-page extraction utilities.
- CJK paragraph reflow helper for cleaning PDF-extracted text before conversion.

## Supported Conversion Configurations

- `s2t`, `t2s`, `s2tw`, `tw2s`, `s2twp`, `tw2sp`, `s2hk`, `hk2s`, `t2tw`, `tw2t`, `t2twp`, `tw2tp`, `t2hk`, `hk2t`,
  `t2jp`, `jp2t`

## Installation

### 1. Install from PyPI

```bash
pip install opencc-pyo3
```

### 2. Build and install the Python wheel using [maturin](https://github.com/PyO3/maturin):

```sh
# In project root
maturin build --release
pip install ./target/wheels/opencc_pyo3-<version>-cp<pyver>-abi3-<platform>.whl
```

Or for development (May require venv):

```sh
maturin develop -r
```

See [build.txt](https://github.com/laisuk/opencc_pyo3/blob/master/build.txt) for detailed build and install
instructions.

## Usage

### Python

```python
from opencc_pyo3 import OpenCC

text = "“春眠不觉晓，处处闻啼鸟。”"
opencc = OpenCC("s2t")
converted = opencc.convert(text, punctuation=True)
print(converted)  # 「春眠不覺曉，處處聞啼鳥。」
```

---

### CLI

You can also use the CLI interface via Python module or Python script:  
Sub-Commands are:

- `convert`: Convert Chinese text using OpenCC
- `office`: Convert Office document Chinese text using OpenCC
- `pdf`: Convert extracted PDF document text using OpenCC

---

#### convert

```bash
python -m opencc_pyo3 convert --help
usage: opencc-pyo3 convert [-h] [-i <file>] [-o <file>] [-c <conversion>] [-p] [--in-enc <encoding>] [--out-enc <encoding>]

options:
  -h, --help            show this help message and exit
  -i, --input <file>    Read original text from <file>.
  -o, --output <file>   Write converted text to <file>.
  -c, --config <conversion>
                        Conversion configuration: s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp
  -p, --punct           Enable punctuation conversion. (Default: False)
  --in-enc <encoding>   Encoding for input. (Default: UTF-8)
  --out-enc <encoding>  Encoding for output. (Default: UTF-8)
```

---

#### office

Support OpenOffice documents and Epub (`.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`, `.epub`)

```bash
python -m opencc_pyo3 office --help                                         
usage: opencc-pyo3 office [-h] [-i <file>] [-o <file>] [-c <conversion>] [-p] [-f <format>] [--auto-ext] [--keep-font]

options:
  -h, --help            show this help message and exit
  -i, --input <file>    Input Office document from <file>.
  -o, --output <file>   Output Office document to <file>.
  -c, --config <conversion>
                        conversion: s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp
  -p, --punct           Enable punctuation conversion. (Default: False)
  -f, --format <format>
                        Target Office format (e.g., docx, xlsx, pptx, odt, ods, odp, epub)
  --auto-ext            Auto-append extension to output file
  --keep-font           Preserve font-family information in Office content
```

---

#### PDF

Support PDF files as input, with built-in text extraction and OpenCC-based conversion powered by `opencc-fmmseg`
(available since v0.8.4).

This command allows you to extract Chinese text from PDF documents, optionally apply CJK-aware paragraph reflow,
and convert the result using OpenCC configurations.

> **Note**  
> Only text-embedded (searchable) PDF documents are supported.  
> Scanned or image-only PDFs without an embedded text layer are not currently supported.

```bash
python -m opencc_pyo3 pdf --help

usage: __main__.py pdf [-h] -i <file> [-o <file>] [-c <conversion>] [-p] [-H] [-r] [--compact] [--timing] [-e]

options:
  -h, --help            show this help message and exit
  -i, --input <file>    Input PDF file.
  -o, --output <file>   Output text file (UTF-8). If omitted, defaults to "<input>_converted.txt".
  -c, --config <conversion>
                        Conversion configuration: s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp
  -p, --punct           Enable punctuation conversion. (Default: False)
  -H, --header          Preserve page-break-like gaps when reflowing CJK paragraphs (passed as add_pdf_page_header to reflow_cjk_paragraphs).
  -r, --reflow          Enable CJK-aware paragraph reflow before conversion.
  --compact             Use compact paragraph mode (single newline between paragraphs).
  --timing              Show time use for each process workflow.
  -e, --extract         Extract PDF text only (skip OpenCC conversion).
```

```sh
python -m opencc_pyo3 convert -i input.txt -o output.txt -c s2t --punct

python -m opencc_pyo3 office -c s2t --punct -i input.docx -o output.docx --keep-font

opencc-pyo3 office -c s2tw -p -i input.epub -o output.epub

opencc-pyo3 pdf -i input.pdf -o output.txt -c s2t -punct --reflow
```

---

## Python API

### `OpenccConfig`

`OpenccConfig` is an enum exported from `opencc_pyo3` for config-safe usage:

```python
from opencc_pyo3 import OpenCC, OpenccConfig

cc = OpenCC(OpenccConfig.S2TW)
print(cc.convert("汉字"))  # 漢字
```

Available enum values:

- `OpenccConfig.S2T`
- `OpenccConfig.T2S`
- `OpenccConfig.S2TW`
- `OpenccConfig.TW2S`
- `OpenccConfig.S2TWP`
- `OpenccConfig.TW2SP`
- `OpenccConfig.S2HK`
- `OpenccConfig.HK2S`
- `OpenccConfig.T2TW`
- `OpenccConfig.TW2T`
- `OpenccConfig.T2TWP`
- `OpenccConfig.TW2TP`
- `OpenccConfig.T2HK`
- `OpenccConfig.HK2T`
- `OpenccConfig.T2JP`
- `OpenccConfig.JP2T`

### `OpenCC`

Core converter class backed by the Rust extension module.

- `OpenCC(config: str | OpenccConfig = "s2t")`
    - Creates a converter using a config string or `OpenccConfig` enum.
    - Invalid config values fall back to `s2t`.
- `convert(input_text: str, punctuation: bool = False) -> str`
    - Converts text using the current config.
- `set_config(config: str | OpenccConfig) -> None`
    - Changes the active config.
- `get_config() -> str`
    - Returns the current canonical config name.
- `get_last_error() -> str`
    - Returns the most recent config error message, or `""` if none.
- `zho_check(input_text: str) -> int`
    - Detects the text type.
    - `1` = Traditional Chinese, `2` = Simplified Chinese, `0` = other / undetermined
- `OpenCC.supported_configs() -> list[str]`
    - Returns all supported config names.
- `OpenCC.is_valid_config(config: str) -> bool`
    - Validates a config string.

Example:

```python
from opencc_pyo3 import OpenCC, OpenccConfig

cc = OpenCC(OpenccConfig.S2T)
print(cc.get_config())  # s2t
print(cc.convert("汉字", punctuation=True))
print(cc.zho_check("汉字"))  # 2

cc.set_config("t2jp")
print(cc.convert("圖書館"))  # 図書館

print(OpenCC.supported_configs())
print(OpenCC.is_valid_config("s2hk"))  # True
```

### Reflow helper

- `reflow_cjk_paragraphs(text: str, add_pdf_page_header: bool, compact: bool) -> str`
    - Reflows PDF-extracted CJK text by merging broken line wraps while preserving paragraph structure.
    - `add_pdf_page_header=True` keeps explicit page-gap style boundaries.
    - `compact=True` uses single newlines between paragraphs.

Example:

```python
from opencc_pyo3.opencc_pyo3 import reflow_cjk_paragraphs

raw_text = "我来到\n无人海边。"
clean_text = reflow_cjk_paragraphs(raw_text, add_pdf_page_header=False, compact=False)  # 我来到无人海边。
```

### PDF extraction APIs

The package currently exposes two PDF extraction layers:

1. Rust extension functions in `opencc_pyo3.opencc_pyo3`
2. PDFium-based helpers in `opencc_pyo3.pdfium_helper`

#### Legacy Rust PDF extract functions

These are still exported, but the type stub marks them as deprecated in favor of the PDFium helper module.

- `extract_pdf_text(path: str) -> str`
- `extract_pdf_text_pages(path: str) -> list[str]`
- `extract_pdf_pages_with_callback(path: str, callback: Callable[[int, int, str], Any]) -> None`

#### Recommended PDFium helper functions

Import from `opencc_pyo3.pdfium_helper`:

- `extract_pdf_pages_with_callback_pdfium(path: str, callback, add_page_header: bool = False) -> None`
- `extract_pdf_text_pdfium_progress(path: str) -> str`
- `extract_pdf_text_pdfium_silent(path: str) -> str`
- `extract_pdf_text_pages_pdfium(path: str) -> list[str]`
- `extract_pdf_text_pages_pdfium_progress(path: str) -> list[str]`
- `make_progress_collector() -> tuple[callback, list[str]]`
- `make_silent_collector() -> tuple[callback, list[str]]`

Example:

```python
from opencc_pyo3 import OpenCC
from opencc_pyo3.opencc_pyo3 import reflow_cjk_paragraphs
from opencc_pyo3.pdfium_helper import extract_pdf_text_pdfium_silent

raw = extract_pdf_text_pdfium_silent("input.pdf")
text = reflow_cjk_paragraphs(raw, add_pdf_page_header=False, compact=False)
converted = OpenCC("s2t").convert(text, punctuation=True)
```

## Development

- Rust source: [src/lib.rs](https://github.com/laisuk/opencc_pyo3/blob/master/src/lib.rs)
- Python bindings: [opencc_pyo3/__init
  __.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__init__.py), [opencc_pyo3/opencc_pyo3.pyi](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/opencc_pyo3.pyi)
- CLI: [opencc_pyo3/__main__.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__main__.py)

## Benchmarks

Latest benchmark results for the optimized current `opencc_pyo3` version.
These replace the much older `v0.7.0` numbers.

```
Package: opencc_pyo3
Python: 3.13.13
Platform: Windows-11-10.0.26200-SP0
Processor: Intel64 Family 6 Model 191 Stepping 2, GenuineIntel
Configs: s2t, s2tw, s2twp
Text sizes: 100, 1,000, 10,000, 100,000 characters
```

### BENCHMARK RESULTS

---

| Method         | Config | TextSize |     Mean |   StdDev |      Min |      Max | Ops/sec |  Chars/sec |
|:---------------|:-------|---------:|---------:|---------:|---------:|---------:|--------:|-----------:|
| Convert_Small  | s2t    |      100 | 0.005 ms | 0.003 ms | 0.004 ms | 0.021 ms | 188,442 | 18,844,221 |
| Convert_Medium | s2t    |    1,000 | 0.038 ms | 0.006 ms | 0.036 ms | 0.066 ms |  26,189 | 26,189,437 |
| Convert_Large  | s2t    |   10,000 | 0.253 ms | 0.093 ms | 0.171 ms | 0.629 ms |   3,958 | 39,577,314 |
| Convert_XLarge | s2t    |  100,000 | 1.394 ms | 0.166 ms | 1.156 ms | 1.699 ms |     717 | 71,726,750 |
| Convert_Small  | s2tw   |      100 | 0.006 ms | 0.003 ms | 0.005 ms | 0.021 ms | 175,953 | 17,595,308 |
| Convert_Medium | s2tw   |    1,000 | 0.044 ms | 0.005 ms | 0.042 ms | 0.071 ms |  22,808 | 22,808,485 |
| Convert_Large  | s2tw   |   10,000 | 0.318 ms | 0.086 ms | 0.227 ms | 0.514 ms |   3,141 | 31,411,310 |
| Convert_XLarge | s2tw   |  100,000 | 1.503 ms | 0.129 ms | 1.355 ms | 1.837 ms |     665 | 66,516,340 |
| Convert_Small  | s2twp  |      100 | 0.008 ms | 0.003 ms | 0.007 ms | 0.025 ms | 130,435 | 13,043,478 |
| Convert_Medium | s2twp  |    1,000 | 0.054 ms | 0.006 ms | 0.052 ms | 0.084 ms |  18,378 | 18,377,849 |
| Convert_Large  | s2twp  |   10,000 | 0.482 ms | 0.249 ms | 0.335 ms | 1.602 ms |   2,075 | 20,746,888 |
| Convert_XLarge | s2twp  |  100,000 | 1.817 ms | 0.197 ms | 1.649 ms | 2.581 ms |     550 | 55,032,341 |

---

### Reproduce Benchmarks

```bash
python bench/opencc_benchmark_md.py --ci --configs s2t s2tw s2twp --sizes Small Medium Large XLarge --export md json --output-dir bench/out
```

## Projects That Use `opencc-pyo3`

[OpenccPyo3Gui](https://github.com/laisuk/OpenccPyo3Gui)

---

## License

[MIT](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)

---

Powered by **Rust**, **PyO3**, **OpenCC**, **Pdfium** and **opencc-fmmseg**.