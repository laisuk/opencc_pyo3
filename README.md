# opencc_pyo3

[![PyPI version](https://img.shields.io/pypi/v/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![Downloads](https://pepy.tech/badge/opencc-pyo3)](https://pepy.tech/project/opencc-pyo3)
[![Python Versions](https://img.shields.io/pypi/pyversions/opencc-pyo3.svg)](https://pypi.org/project/opencc-pyo3/)
[![License](https://img.shields.io/github/license/laisuk/opencc_pyo3)](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)
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

- `s2t`, `t2s`, `s2tw`, `tw2s`, `s2twp`, `tw2sp`, `s2hk`, `hk2s`, `s2hkp`, `hk2sp`, `t2tw`, `tw2t`, `t2twp`,
  `tw2tp`, `t2hk`, `t2hkp`, `hk2t`, `hk2tp`, `t2jp`, `jp2t`

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
usage: opencc-pyo3 convert [-h] [-i <file>] [-o <file>] [-c <conversion>] [-p] [-n] [--detofu [<level>]] [--detofu-file <file>] [-D <slot:mode:path>]
                           [--in-enc <encoding>] [--out-enc <encoding>]

optional arguments:
  -h, --help            show this help message and exit
  -i <file>, --input <file>
                        Read original text from <file>. (default: None)
  -o <file>, --output <file>
                        Write converted text to <file>. (default: None)
  -c <conversion>, --config <conversion>
                        Configuration: s2t|s2tw|s2twp|s2hk|s2hkp|t2s|t2tw|t2twp|t2hk|t2hkp|tw2s|tw2sp|tw2t|tw2tp|hk2s|hk2sp|hk2t|hk2tp|jp2t|t2jp (default: None)
  -p, --punct           Enable punctuation conversion. (Default: False) (default: False)
  -n, --norm-compat     Normalize CJK Compatibility Ideographs before conversion. (Default: False) (default: False)
  --detofu [<level>]    Apply tofu-safe fallback after conversion. Levels: all/ExtB, ExtC, ExtD, ExtE, ExtF, ExtG, ExtH, ExtI. (default: None)
  --detofu-file <file>  Load additional detofu fallback mappings from a UTF-8 text file. Custom mappings override built-in mappings; requires --detofu.
                        (default: None)
  -D <slot:mode:path>, --custom-dict <slot:mode:path>
                        Load custom dictionary file. Format: slot:mode:path, e.g. STPhrases:append:custom.txt. Can be used multiple times.
                        Available slots: STCharacters|STPhrases|STPunctuations|TSCharacters|TSPhrases|TSPunctuations|TWPhrases|TWPhrasesRev|HKPhrases|HKPhrasesRev|TWVariants|TWVariantsPhrases|TWVariantsRev|TWVariantsRevPhrases|HKVariants|HKVariantsPhrases|HKVariantsRev|HKVariantsRevPhrases|JPSCharacters|JPSCharactersRev|JPSPhrases (default: None)
  --in-enc <encoding>   Encoding for input. (Default: UTF-8) (default: UTF-8)
  --out-enc <encoding>  Encoding for output. (Default: UTF-8) (default: UTF-8)
```

---

#### office

Support OpenOffice documents and Epub (`.docx`, `.xlsx`, `.pptx`, `.odt`, `.ods`, `.odp`, `.epub`)

```bash
python -m opencc_pyo3 office --help                                         
usage: opencc-pyo3 office [-h] [-i <file>] [-o <file>] [-c <conversion>] [-p] [-f <format>] [-k] [-D <slot:mode:path>]                                

optional arguments:
  -h, --help            show this help message and exit
  -i <file>, --input <file>
                        Input Office document from <file>. (default: None)
  -o <file>, --output <file>
                        Output Office document to <file>. (default: None)
  -c <conversion>, --config <conversion>
                        Configuration: s2t|s2tw|s2twp|s2hk|s2hkp|t2s|t2tw|t2twp|t2hk|t2hkp|tw2s|tw2sp|tw2t|tw2tp|hk2s|hk2sp|hk2t|hk2tp|jp2t|t2jp (default: None)
  -p, --punct           Enable punctuation conversion. (Default: False) (default: False)
  -f <format>, --format <format>
                        Target Office format (e.g., docx, xlsx, pptx, odt, ods, odp, epub) (default: None)
  -k, --keep-font       Preserve font-family information in Office content (default: False)
  -D <slot:mode:path>, --custom-dict <slot:mode:path>
                        Load custom dictionary file. Format: slot:mode:path, e.g. STPhrases:append:custom.txt. Can be used multiple times.
                        Available slots: STCharacters|STPhrases|STPunctuations|TSCharacters|TSPhrases|TSPunctuations|TWPhrases|TWPhrasesRev|HKPhrases|HKPhrasesRev|TWVariants|TWVariantsPhrases|TWVariantsRev|TWVariantsRevPhrases|HKVariants|HKVariantsPhrases|HKVariantsRev|HKVariantsRevPhrases|JPSCharacters|JPSCharactersRev|JPSPhrases (default: None)
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
usage: opencc-pyo3 pdf [-h] -i <file> [-o <file>] [-c <conversion>] [-p] [-H] [-r] [-C] [--timing] [-e] [-n] [-D <slot:mode:path>]

optional arguments:
  -h, --help            show this help message and exit
  -i <file>, --input <file>
                        Input PDF file. (default: None)
  -o <file>, --output <file>
                        Output text file (UTF-8). If omitted, defaults to "<input>_converted.txt". (default: None)
  -c <conversion>, --config <conversion>
                        Configuration: s2t|s2tw|s2twp|s2hk|s2hkp|t2s|t2tw|t2twp|t2hk|t2hkp|tw2s|tw2sp|tw2t|tw2tp|hk2s|hk2sp|hk2t|hk2tp|jp2t|t2jp (default: None)
  -p, --punct           Enable punctuation conversion. (Default: False) (default: False)
  -H, --header          Preserve page-break-like gaps when reflowing CJK paragraphs (passed as add_pdf_page_header to reflow_cjk_paragraphs). (default: False)
  -r, --reflow          Enable CJK-aware paragraph reflow before conversion. (default: False)
  -C, --compact         Use compact paragraph mode (single newline between paragraphs). (default: False)
  --timing              Show time use for each process workflow. (default: False)
  -e, --extract         Extract PDF text only (skip OpenCC conversion). (default: False)
  -n, --norm-compat     Normalize CJK Compatibility Ideographs before conversion. (Default: False) (default: False)
  -D <slot:mode:path>, --custom-dict <slot:mode:path>
                        Load custom dictionary file. Format: slot:mode:path, e.g. STPhrases:append:custom.txt. Can be used multiple times.
                        Available slots: STCharacters|STPhrases|STPunctuations|TSCharacters|TSPhrases|TSPunctuations|TWPhrases|TWPhrasesRev|HKPhrases|HKPhrasesRev|TWVariants|TWVariantsPhrases|TWVariantsRev|TWVariantsRevPhrases|HKVariants|HKVariantsPhrases|HKVariantsRev|HKVariantsRevPhrases|JPSCharacters|JPSCharactersRev|JPSPhrases (default: None)
```

```sh
python -m opencc_pyo3 convert -i input.txt -o output.txt -c s2t --punct

python -m opencc_pyo3 convert -i input.txt -o output.txt -c t2s --norm-compat

python -m opencc_pyo3 convert -i input.txt -o output.txt -c s2t --detofu all --detofu-file custom_detofu.txt

echo "這個細路哥很靈活" | python -m opencc_pyo3 convert -c hk2sp --custom-dict HKVariantsRevPhrases:append:my_hk_dict.txt
# output: 这个小男孩很灵活

echo "天龍八部書裡的喬峰是契丹人" | opencc-pyo3 convert -c t2s --norm-compat
# output: 天龙八部书里的乔峰是契丹人
 
python -m opencc_pyo3 office -c s2t --punct -i input.docx -o output.docx --keep-font

opencc-pyo3 office -c s2tw -p -i input.epub -o output.epub

opencc-pyo3 pdf -i input.pdf -o output.txt -c s2t --punct --reflow --norm-compat
```

my_hk_dict.txt:

`--custom-dict` accepts `slot:mode:path` and can be passed more than once. The token is validated before conversion:
`slot`, `mode`, and `path` must all be present. Supported merge modes are `append` and `override`; common slots include
`STPhrases`, `TWPhrases`, `HKVariantsRevPhrases`, and `JPSCharactersRev`.

```text
細路哥	小男孩
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
- `OpenccConfig.T2HKP`
- `OpenccConfig.HK2T`
- `OpenccConfig.HK2TP`
- `OpenccConfig.T2JP`
- `OpenccConfig.JP2T`
- `OpenccConfig.S2HKP`
- `OpenccConfig.HK2SP`

### `OpenCC`

Core converter class backed by the Rust extension module.

- `OpenCC(config: str | OpenccConfig = "s2t", preserve_ids: bool = False)`
    - `config`: OpenCC conversion configuration. Invalid values fall back to `s2t`.
    - `preserve_ids`: Preserve characters inside Unicode IDS (Ideographic Description Sequence) structures such as `⿰`,
      `⿱`, and `⿲` during conversion. Default is `False`.
- `OpenCC.from_dicts(config="s2t", specs=None) -> OpenCC`
    - Creates a converter with programmatic, in-memory custom dictionary entries.
- `OpenCC.from_dict_files(config="s2t", specs=None) -> OpenCC`
    - Creates a converter with OpenCC-style custom dictionary files.
- `convert(input_text: str, punctuation: bool = False) -> str`
    - Converts text using the current config.
- `detofu(text: str, level: str = "all") -> str`
    - Replaces rare CJK extension characters with display-safe fallback characters.
- `detofu_with_custom_file(text: str, level: str = "all", path: str) -> str`
    - Applies DeTofu with additional UTF-8 fallback mappings from a file.
- `detofu_with_custom_pairs(text: str, level: str = "all", pairs: list[tuple[str, str]]) -> str`
    - Applies DeTofu with additional in-memory fallback character pairs.
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
- `OpenCC.available_slots() -> list[str]`
    - Returns all 21 active canonical custom dictionary slot names.

Example:

```python
from opencc_pyo3 import OpenCC, OpenccConfig

cc = OpenCC(OpenccConfig.S2T)
print(cc.get_config())  # s2t
print(cc.convert("汉字", punctuation=True))
print(cc.zho_check("汉字"))  # 2

cc = OpenCC("t2s", preserve_ids=True)
print(cc.convert("漢字 ⿰氵漢 馬"))  # 汉字 ⿰氵漢 马

cc.set_config("t2jp")
print(cc.convert("圖書館"))  # 図書館

# Phrase-aware Traditional ↔ Hong Kong Traditional conversions
print(OpenCC(OpenccConfig.T2HKP).get_config())  # t2hkp
print(OpenCC(OpenccConfig.HK2TP).get_config())  # hk2tp

print(OpenCC.supported_configs())
print(OpenCC.available_slots())
print(OpenCC.is_valid_config("s2hk"))  # True
```

### DeTofu Display Fallbacks

DeTofu can replace rare CJK extension characters with more widely displayable fallback characters after normal OpenCC
conversion. The canonical argument order is always input text, level, then custom source. `level="all"` is equivalent to
`ExtB` and replaces ExtB and later extension characters; compact and CLI-friendly spellings such as `B`, `ExtC`, and
`ext-c` are accepted.

```python
from opencc_pyo3 import OpenCC

cc = OpenCC("s2t")

print(cc.detofu("𠉂𪠟𫝈𫬐", "all"))
print(cc.detofu_with_custom_file("𣭲毛", "all", "custom_detofu.txt"))
print(cc.detofu_with_custom_pairs("𣭲毛", "all", [("𣭲", "氄")]))
```

Custom DeTofu files use one mapping per line:

```text
𣭲	氄	B
```

---

### Custom Dictionaries

`OpenCC("s2t")` remains the recommended API for normal use and continues to use the built-in embedded dictionaries.
Use custom dictionaries only when you need project-specific terms or overrides.

Custom dictionaries are applied during construction. The backend first loads the default embedded zstd dictionaries,
then
applies post-load customization with `DictionaryMaxlength::from_zstd()?.with_custom_dicts(...)` or
`DictionaryMaxlength::from_zstd()?.with_custom_dict_files(...)`. The final `OpenCC` instance remains immutable and
optimized after construction. Runtime hot reload is not supported; rebuild a new `OpenCC` instance if dictionaries need
to change.

#### In-memory custom dictionaries

Use `OpenCC.from_dicts()` for programmatic terms:

```python
from typing import List

from opencc_pyo3 import OpenCC, CustomDictSpec

specs: List[CustomDictSpec] = [
    {
        "slot": "STPhrases",
        "pairs": [("帕兰蒂尔", "柏蘭蒂爾")],
        "mode": "append",
    }
]

cc = OpenCC.from_dicts("s2t", specs)

print(cc.convert("帕兰蒂尔是一家公司"))
# 柏蘭蒂爾是一家公司
```

#### File-based custom dictionaries

Use `OpenCC.from_dict_files()` for OpenCC-style dictionary files:

```python
from typing import List

from opencc_pyo3 import OpenCC, CustomDictFileSpec

specs: List[CustomDictFileSpec] = [
    {
        "slot": "STPhrases",
        "files": ["custom_st_phrases.txt"],
        "mode": "append",
    }
]

cc = OpenCC.from_dict_files("s2t", specs)
```

Custom dictionary files use one mapping per line:

```text
source<TAB>target
```

Example:

```text
帕兰蒂尔	柏蘭蒂爾
```

#### Merge modes

- `append`: add custom entries to the existing dictionary slot. Duplicate keys follow the backend "last wins" behavior.
- `override`: replace the entire target dictionary slot with the custom entries or files.

#### Supported dictionary slots

Use canonical slot names without `.txt`, such as `STPhrases`, not `STPhrases.txt`. The Python wrapper may tolerate
`.txt`, but the documented API uses canonical names only.

| Slot                   | Purpose                                             | OpenCC dictionary file         |
|:-----------------------|:----------------------------------------------------|:-------------------------------|
| `STCharacters`         | Simplified → Traditional character mappings         | `STCharacters.txt`             |
| `STPhrases`            | Simplified → Traditional phrase mappings            | `STPhrases.txt`                |
| `STPunctuations`       | Simplified → Traditional punctuation mappings       | `STPunctuations.txt`           |
| `TSCharacters`         | Traditional → Simplified character mappings         | `TSCharacters.txt`             |
| `TSPhrases`            | Traditional → Simplified phrase mappings            | `TSPhrases.txt`                |
| `TSPunctuations`       | Traditional → Simplified punctuation mappings       | `TSPunctuations.txt`           |
| `TWPhrases`            | Traditional → Taiwan phrase mappings                | `TWPhrases.txt`                |
| `TWPhrasesRev`         | Taiwan → Traditional reverse phrase mappings        | `TWPhrasesRev.txt`             |
| `HKPhrases`            | Traditional → Hong Kong phrase mappings             | `HKPhrases.txt`                |
| `HKPhrasesRev`         | Hong Kong → Traditional reverse phrase mappings     | `HKPhrasesRev.txt`             |
| `TWVariants`           | Traditional → Taiwan regional character variants    | `TWVariants.txt`               |
| `TWVariantsPhrases`    | Traditional → Taiwan regional phrase variants       | `TWVariantsPhrases.txt`        |
| `TWVariantsRev`        | Taiwan → Traditional reverse character variants     | `TWVariantsRev.txt`            |
| `TWVariantsRevPhrases` | Taiwan → Traditional reverse phrase variants        | `TWVariantsRevPhrases.txt`     |
| `HKVariants`           | Traditional → Hong Kong regional character variants | `HKVariants.txt`               |
| `HKVariantsPhrases`    | Traditional → Hong Kong regional phrase variants    | `HKVariantsPhrases.txt`        |
| `HKVariantsRev`        | Hong Kong → Traditional reverse character variants  | `HKVariantsRev.txt`            |
| `HKVariantsRevPhrases` | Hong Kong → Traditional reverse phrase variants     | `HKVariantsRevPhrases.txt`     |
| `JPSCharacters`        | Japanese Shinjitai character mappings               | `JPShinjitaiCharacters.txt`    |
| `JPSCharactersRev`     | Japanese Shinjitai reverse character mappings       | `JPShinjitaiCharactersRev.txt` |
| `JPSPhrases`           | Japanese Shinjitai phrase mappings                  | `JPShinjitaiPhrases.txt`       |

Custom dictionary behavior follows the same OpenCC dictionary-slot model. Choosing the wrong slot may have no effect or
may affect a different conversion path. For `s2t`, use `STCharacters` or `STPhrases`. For `t2s`, use `TSCharacters` or
`TSPhrases`. For Taiwan regional behavior, use `TWPhrases`, `TWPhrasesRev`, `TWVariantsPhrases`, `TWVariants`,
`TWVariantsRev`, or `TWVariantsRevPhrases`. For Hong Kong regional behavior, use `HKPhrases`, `HKPhrasesRev`,
`HKVariantsPhrases`, `HKVariants`, `HKVariantsRev`, or `HKVariantsRevPhrases`. For Japanese Shinjitai behavior, use
`JPSCharacters`, `JPSCharactersRev`, or `JPSPhrases`.

#### Typing helpers

`CustomDictSpec` and `CustomDictFileSpec` are exported for typed Python code:

```python
from typing import List
from opencc_pyo3 import OpenCC, CustomDictSpec

specs: List[CustomDictSpec] = [
    {
        "slot": "STPhrases",
        "pairs": [("帕兰蒂尔", "柏蘭蒂爾")],
        "mode": "append",
    }
]

cc = OpenCC.from_dicts("s2t", specs)
```

Notes:

- Custom dictionaries are loaded at construction time.
- Existing `OpenCC` objects are immutable after construction.
- Runtime hot reload is not supported.
- Rebuild a new `OpenCC` instance if dictionaries need to change.
- Invalid slots, invalid modes, malformed lines, or unreadable files raise errors.
- `OpenCC("s2t")` remains the recommended API for normal users.
- Use `from_dicts()` for programmatic or in-memory custom terms.
- Use `from_dict_files()` for OpenCC-style dictionary files.

---

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

The package exposes PDFium-based PDF extraction helpers in `opencc_pyo3.pdfium_helper`.

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
- Python bindings: [opencc_pyo3/__init__.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__init__.py), [opencc_pyo3/opencc_pyo3.pyi](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/opencc_pyo3.pyi)
- CLI: [opencc_pyo3/__main__.py](https://github.com/laisuk/opencc_pyo3/blob/master/opencc_pyo3/__main__.py)

## Benchmarks

Historical benchmark results for `opencc-pyo3` v0.10.2, collected by the
[Benchmark on macos-latest workflow](https://github.com/laisuk/opencc_pyo3/actions/runs/29220806465#summary-86725382175).

```
Package: opencc_pyo3
Version: 0.10.2
OS: macOS 26.4 (arm64)
CPU: Apple M1 (Virtual)
Cores: 3
Memory: 7.0 GiB
Python: 3.10.11
Rust: rustc 1.97.0 (2d8144b78 2026-07-07)
Configs: s2t, s2tw, s2twp
Text sizes: 100, 1,000, 10,000, 100,000 characters
```

### BENCHMARK RESULTS

---

| Method         | Config | Text Size | Mean (ms) | Median (ms) | StdDev (ms) | Min (ms) | Max (ms) | Ops/sec |  Chars/sec |
|:---------------|:-------|----------:|----------:|------------:|------------:|---------:|---------:|--------:|-----------:|
| Convert_Small  | s2t    |       100 |     0.008 |       0.005 |       0.011 |    0.004 |    0.071 | 128,239 | 12,823,911 |
| Convert_Medium | s2t    |     1,000 |     0.038 |       0.037 |       0.008 |    0.036 |    0.108 |  26,113 | 26,113,217 |
| Convert_Large  | s2t    |    10,000 |     0.236 |       0.237 |       0.030 |    0.190 |    0.337 |   4,229 | 42,287,025 |
| Convert_XLarge | s2t    |   100,000 |     2.144 |       2.088 |       0.285 |    1.751 |    3.034 |     466 | 46,638,386 |
| Convert_Small  | s2tw   |       100 |     0.005 |       0.005 |       0.001 |    0.005 |    0.011 | 193,679 | 19,367,928 |
| Convert_Medium | s2tw   |     1,000 |     0.046 |       0.043 |       0.010 |    0.043 |    0.133 |  21,755 | 21,754,749 |
| Convert_Large  | s2tw   |    10,000 |     0.311 |       0.309 |       0.037 |    0.237 |    0.424 |   3,219 | 32,190,775 |
| Convert_XLarge | s2tw   |   100,000 |     2.949 |       2.860 |       0.445 |    2.282 |    4.753 |     339 | 33,914,979 |
| Convert_Small  | s2twp  |       100 |     0.006 |       0.006 |       0.000 |    0.005 |    0.011 | 177,416 | 17,741,637 |
| Convert_Medium | s2twp  |     1,000 |     0.048 |       0.048 |       0.002 |    0.047 |    0.061 |  20,783 | 20,782,517 |
| Convert_Large  | s2twp  |    10,000 |     0.333 |       0.343 |       0.037 |    0.264 |    0.544 |   3,002 | 30,021,166 |
| Convert_XLarge | s2twp  |   100,000 |     3.034 |       3.024 |       0.273 |    2.497 |    3.844 |     330 | 32,964,999 |

### Summary

For representative Medium through XLarge inputs, `s2t` is consistently the fastest configuration, reaching about
46.6 million characters per second for the 100,000-character sample. The regional `s2tw` and phrase-aware `s2twp`
pipelines perform additional dictionary work and reach about 33.9 million and 33.0 million characters per second,
respectively, at the same size. Mean and median timings are now close on the larger samples, indicating that full-input
warmups and shuffled execution substantially reduced the earlier CI outlier distortion. Results for the 100-character
sample should still be treated as microbenchmark-scale measurements, where timer and scheduling noise have a larger
relative effect.

---

### Reproduce Benchmarks

```bash
python bench/opencc_benchmark_md.py --ci --configs s2t s2tw s2twp --sizes Small Medium Large XLarge --iterations 120 --warmup 15 --warmup-full-text --shuffle-order --export md json --output-dir bench/out
```

## Projects That Use `opencc-pyo3`

[OpenccPyo3Gui](https://github.com/laisuk/OpenccPyo3Gui)

---

## License

[MIT](https://github.com/laisuk/opencc_pyo3/blob/master/LICENSE)

---

Powered by **Rust**, **PyO3**, **OpenCC**, **Pdfium** and **opencc-fmmseg**.
