# use_pdfium_progress_collector.py
from __future__ import annotations

import argparse
import sys
from pathlib import Path

from opencc_pyo3.opencc_pyo3 import reflow_cjk_paragraphs
from opencc_pyo3.pdfium_helper import extract_pdf_text_pdfium_progress


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Extract PDF with PDFium (progress), then reflow CJK paragraphs."
    )
    parser.add_argument(
        "-i", "--input",
        default="盗墓笔记.pdf",
        help="Input PDF path",
    )
    parser.add_argument(
        "-o", "--output",
        default="盗墓笔记_extracted.txt",
        help="Output text path",
    )
    parser.add_argument(
        "--add-pdf-page-header",
        action="store_true",
        help="Add page headers during reflow",
    )
    parser.add_argument(
        "--compact",
        action="store_true",
        help="Compact reflow mode",
    )
    args = parser.parse_args()

    input_path = Path(args.input)
    if not input_path.is_file():
        print("❌ PDF file not found.", file=sys.stderr)
        print(f"  Path : {input_path}", file=sys.stderr)
        return 2

    print(f"Extracting PDF page-by-page with PDFium: {input_path.name}")

    # ✅ One-liner: extract + progress
    full_text = extract_pdf_text_pdfium_progress(str(input_path))

    print(f"Total extracted characters: {len(full_text):,}")

    print("Reflowing CJK paragraphs...")
    reflowed = reflow_cjk_paragraphs(
        full_text,
        add_pdf_page_header=args.add_pdf_page_header,
        compact=args.compact,
    )

    out_path = Path(args.output)
    print(f"Writing reflowed text to: {out_path}")
    out_path.write_text(reflowed, encoding="utf-8", newline="\n")

    print("Done.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
