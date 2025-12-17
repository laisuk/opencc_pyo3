# use_pdfium_progress.py
from __future__ import annotations
from typing import List
from opencc_pyo3 import (
    extract_pdf_pages_with_callback_pdfium,
    reflow_cjk_paragraphs,
)

# input_file = "tests/My_Golden_Blood.pdf"
input_file = "tests/盗墓笔记.pdf"
# output_file = "tests/My_Golden_Blood_extracted.txt"
output_file = "tests/盗墓笔记_extracted.txt"

_pages: List[str] = []


def on_page(page: int, total: int, text: str) -> None:
    percent = page * 100 // total
    msg = f"[{page}/{total}] ({percent:3d}%) Extracted {len(text)} chars"
    # pad with spaces so previous content is fully overwritten
    print(msg.ljust(80), end="\r", flush=True)
    _pages.append(text)


def main() -> None:
    print(f"Extracting PDF page-by-page with PDFium: {input_file}")

    # PDFium backend: if it fails, it's usually a fatal PDF issue
    extract_pdf_pages_with_callback_pdfium(input_file, on_page)

    full_text = "".join(_pages)
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
