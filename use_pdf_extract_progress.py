# use_pdf_extract_progress.py
from __future__ import annotations
from typing import List
from opencc_pyo3 import (
    extract_pdf_pages_with_callback,
    reflow_cjk_paragraphs,
)

input_file = "tests/My_Golden_Blood.pdf"
# input_file = "tests/盗墓笔记.pdf"
output_file = "tests/My_Golden_Blood_extracted.txt"
# output_file = "tests/盗墓笔记_extracted.txt"

_pages: List[str] = []


def on_page(page: int, total: int, text: str) -> None:
    percent = page * 100 // total
    msg = f"[{page}/{total}] ({percent:3d}%) Extracted {len(text)} chars"
    print(msg.ljust(80), end="\r", flush=True)
    _pages.append(text)

def main() -> None:
    print(f"Extracting PDF page-by-page: {input_file}")

    try:
        extract_pdf_pages_with_callback(input_file, on_page)
    except RuntimeError as exc:
        # Pure-Rust backend cannot handle this file at all (0 chars, 0 pages).
        # For now, just report and exit; this file needs a PDFium engine.
        msg = str(exc)
        print(f"⚠️  pdf-extract backend failed:\n    {msg}")
        print("This PDF likely requires a PDFium-based engine. Skipping reflow.")
        return

    full_text = "".join(_pages)
    print(f"\nTotal extracted characters: {len(full_text)}")

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
