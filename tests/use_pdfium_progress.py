# use_pdfium_progress.py
from __future__ import annotations

import sys
from pathlib import Path
from typing import List
from opencc_pyo3.opencc_pyo3 import (
    reflow_cjk_paragraphs,
)
# PDF extraction (requires pdfium.dll presence)
from opencc_pyo3.pdfium_helper import extract_pdf_pages_with_callback_pdfium, make_progress_collector

input_file = "简体字.pdf"
# input_file = "盗墓笔记.pdf"
output_file = "简体字_extracted.txt"
# output_file = "盗墓笔记_extracted.txt"

_pages: List[str] = []


def on_page(page: int, total: int, text: str) -> None:
    percent = page * 100 // total
    msg = f"Loading [{page}/{total}] ({percent:3d}%) Extracted {len(text)} chars"
    # pad with spaces so previous content is fully overwritten
    print(msg.ljust(80), end="\r", flush=True)
    _pages.append(text)


def main() -> None:
    input_path_str = str(input_file)

    p = Path(input_path_str)
    if not p.is_file():
        print("❌ PDF file not found.", file=sys.stderr)
        print(f"  Path : {input_path_str}", file=sys.stderr)
        return

    print(f"Extracting PDF page-by-page with PDFium: {input_file}")

    callback, pages = make_progress_collector()
    # PDFium backend: if it fails, it's usually a fatal PDF issue
    extract_pdf_pages_with_callback_pdfium(input_file, callback)

    full_text = "".join(pages)
    print()  # move to next line after progress
    print(f"Total extracted characters: {len(full_text):,}")

    print("Reflowing CJK paragraphs...")
    reflowed = reflow_cjk_paragraphs(
        full_text,
        add_pdf_page_header=False,
        compact=False,
    )

    print(f"Writing reflowed text to: {output_file}")
    with open(output_file, "w", encoding="utf-8", newline="\n") as f:
        f.write(reflowed)

    print("Done.")


if __name__ == "__main__":
    main()
