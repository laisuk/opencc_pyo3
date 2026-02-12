from __future__ import print_function

import argparse
import io
import os
import sys

from opencc_pyo3 import OpenCC
from .office_helper import OFFICE_FORMATS, convert_office_doc


def subcommand_convert(args):
    if args.config is None:
        print("ℹ️  Config not specified. Use default 's2t'", file=sys.stderr)
        args.config = 's2t'

    # Plain text conversion fallback
    opencc = OpenCC(args.config)

    # Prompt user if input is from terminal
    if args.input is None and sys.stdin.isatty():
        print("Input text to convert, <Ctrl+Z>/<Ctrl+D> to submit:", file=sys.stderr)

    # Read input text (from file or stdin)
    with io.open(args.input if args.input else 0, encoding=args.in_enc) as f:
        input_str = f.read()

    # Perform conversion
    output_str = opencc.convert(input_str, args.punct)

    # Write output text (to file or stdout)
    with io.open(args.output if args.output else 1, 'w', encoding=args.out_enc) as f:
        f.write(output_str)

    in_from = args.input if args.input else "<stdin>"
    out_to = args.output if args.output else "stdout"
    if sys.stderr.isatty():
        print(f"Conversion completed ({args.config}): {in_from} -> {out_to}", file=sys.stderr)

    return 0


def subcommand_office(args):
    from pathlib import Path

    if args.config is None:
        print("ℹ️  Config not specified. Use default 's2t'", file=sys.stderr)
    args.config = 's2t'

    input_file = args.input
    output_file = args.output
    office_format = args.format
    auto_ext = getattr(args, "auto_ext", False)
    config = args.config
    punct = args.punct
    keep_font = getattr(args, "keep_font", False)

    # Check for missing input/output files
    if not input_file and not output_file:
        print("❌  Input and output files are missing.", file=sys.stderr)
        return 1
    if not input_file:
        print("❌  Input file is missing.", file=sys.stderr)
        return 1

    # If output file is not specified, generate one based on input file
    if not output_file:
        input_path = Path(input_file)

        input_name = input_path.stem
        input_ext = input_path.suffix
        input_dir = input_path.parent if input_path.parent != Path("") else Path.cwd()

        if auto_ext and office_format in OFFICE_FORMATS:
            ext = f".{office_format}"
        else:
            ext = input_ext

        output_path = input_dir / f"{input_name}_converted{ext}"
        output_file = str(output_path)

        print(f"ℹ️  Output file not specified. Using: {output_path}", file=sys.stderr)

    # Determine office format from file extension if not provided
    if not office_format:
        file_ext = os.path.splitext(input_file)[1].lower()
        if file_ext[1:] not in OFFICE_FORMATS:
            print(f"❌  Invalid Office file extension: {file_ext}", file=sys.stderr)
            print("   Valid extensions: .docx | .xlsx | .pptx | .odt | .ods | .odp | .epub", file=sys.stderr)
            return 1
        office_format = file_ext[1:]

    # Auto-append extension to output file if needed
    if auto_ext and output_file and not os.path.splitext(output_file)[1] and office_format in OFFICE_FORMATS:
        output_file += f".{office_format}"
        print(f"ℹ️  Auto-extension applied: {output_file}", file=sys.stderr)

    try:
        # Perform Office document conversion
        success, message = convert_office_doc(
            input_file,
            output_file,
            office_format,
            OpenCC(config),
            punct,
            keep_font,
        )
        if success:
            print(f"{message}\n📁  Output saved to: {os.path.abspath(output_file)}", file=sys.stderr)
            return 0
        else:
            print(f"❌  Conversion failed: {message}", file=sys.stderr)
            return 1
    except Exception as ex:
        print(f"❌  Error during Office document conversion: {str(ex)}", file=sys.stderr)
        return 1


def subcommand_pdf(args) -> int:
    import time
    from pathlib import Path
    from typing import List

    from .opencc_pyo3 import reflow_cjk_paragraphs
    from opencc_pyo3.pdfium_helper import (
        extract_pdf_pages_with_callback_pdfium,
    )

    t0_total = None
    input_path = args.input
    input_path_str = str(input_path)

    p = Path(input_path_str)
    if not p.is_file():
        print("❌ PDF file not found.", file=sys.stderr)
        print(f"  Path : {input_path_str}", file=sys.stderr)
        return 2

    # Determine output filename
    if args.output:
        output_path = args.output
    else:
        stem = str(Path(input_path).with_suffix(""))
        suffix = "_extracted.txt" if args.extract else "_converted.txt"
        output_path = f"{stem}{suffix}"

    if args.timing:
        t0_total = time.perf_counter()

    # ---------------------------------------------------------
    # PDFium extraction (single source of truth)
    # ---------------------------------------------------------
    pages: List[str] = []

    def _on_page(page: int, total: int, chunk: str) -> None:
        percent = page * 100 // total if total else 100
        msg = f"Loading [{page}/{total}] ({percent:3d}%) Extracted {len(chunk)} chars"
        print(msg.ljust(80), end="\r", flush=True)
        pages.append(chunk)

    print(f"Extracting PDF page-by-page with PDFium: {input_path}")
    extract_pdf_pages_with_callback_pdfium(input_path_str, _on_page)
    print()  # newline after progress

    text = "".join(pages)

    if args.timing:
        t1_extract = time.perf_counter()
        print(f"[timing] PDF extract: {(t1_extract - t0_total) * 1000:.1f} ms")

    # ---------------------------------------------------------
    # Reflow (optional)
    # ---------------------------------------------------------
    if args.reflow:
        print("Reflowing CJK paragraphs...")
        text = reflow_cjk_paragraphs(
            text,
            add_pdf_page_header=args.header,
            compact=args.compact,
        )

    # ---------------------------------------------------------
    # OpenCC Conversion (optional)
    # ---------------------------------------------------------
    if not args.extract:
        if args.config:
            opencc = OpenCC(args.config)
            text = opencc.convert(text, args.punct)
    else:
        if args.config or args.punct:
            print("ℹ️  --extract specified: skipping OpenCC conversion.", file=sys.stderr)

    # ---------------------------------------------------------
    # Write Output
    # ---------------------------------------------------------
    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)

    print(f"📄 Input : {input_path}")
    print(f"📁 Output: {output_path}")
    print("⚙️ Engine : pdfium")
    if args.extract:
        print("🧾 Mode  : extract-only (no OpenCC)")
    elif args.config:
        print(f"🧾 Config: {args.config} (punct={'on' if args.punct else 'off'})")

    return 0


def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        description="opencc_pyo3 – Rust/PyO3-based OpenCC CLI",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    # -----------------
    # convert subcommand
    # -----------------
    parser_convert = subparsers.add_parser(
        "convert", help="Convert Chinese text using OpenCC"
    )
    parser_convert.add_argument(
        "-i",
        "--input",
        metavar="<file>",
        help="Read original text from <file>.",
    )
    parser_convert.add_argument(
        "-o",
        "--output",
        metavar="<file>",
        help="Write converted text to <file>.",
    )
    parser_convert.add_argument(
        "-c",
        "--config",
        metavar="<conversion>",
        help=(
            "Conversion configuration: "
            "s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp"
        ),
    )
    parser_convert.add_argument(
        "-p",
        "--punct",
        action="store_true",
        default=False,
        help="Enable punctuation conversion. (Default: False)",
    )
    parser_convert.add_argument(
        "--in-enc",
        metavar="<encoding>",
        default="UTF-8",
        help="Encoding for input. (Default: UTF-8)",
    )
    parser_convert.add_argument(
        "--out-enc",
        metavar="<encoding>",
        default="UTF-8",
        help="Encoding for output. (Default: UTF-8)",
    )
    parser_convert.set_defaults(func=subcommand_convert)

    # -----------------
    # office subcommand
    # -----------------
    parser_office = subparsers.add_parser(
        "office",
        help="Convert Office document and EPUB Chinese text using OpenCC",
    )
    parser_office.add_argument(
        "-i",
        "--input",
        metavar="<file>",
        help="Input Office document from <file>.",
    )
    parser_office.add_argument(
        "-o",
        "--output",
        metavar="<file>",
        help="Output Office document to <file>.",
    )
    parser_office.add_argument(
        "-c",
        "--config",
        metavar="<conversion>",
        help=(
            "conversion: "
            "s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp"
        ),
    )
    parser_office.add_argument(
        "-p",
        "--punct",
        action="store_true",
        default=False,
        help="Enable punctuation conversion. (Default: False)",
    )
    parser_office.add_argument(
        "-f",
        "--format",
        metavar="<format>",
        help="Target Office format (e.g., docx, xlsx, pptx, odt, ods, odp, epub)",
    )
    parser_office.add_argument(
        "--auto-ext",
        action="store_true",
        default=False,
        help="Auto-append extension to output file",
    )
    parser_office.add_argument(
        "--keep-font",
        action="store_true",
        default=False,
        help="Preserve font-family information in Office content",
    )
    parser_office.set_defaults(func=subcommand_office)

    # -------------
    # pdf subcommand
    # -------------
    parser_pdf = subparsers.add_parser(
        "pdf",
        help="Extract + convert Chinese text from a PDF using OpenCC",
    )
    parser_pdf.add_argument(
        "-i",
        "--input",
        metavar="<file>",
        required=True,
        help="Input PDF file.",
    )
    parser_pdf.add_argument(
        "-o",
        "--output",
        metavar="<file>",
        help=(
            "Output text file (UTF-8). "
            'If omitted, defaults to "<input>_converted.txt".'
        ),
    )
    parser_pdf.add_argument(
        "-c",
        "--config",
        metavar="<conversion>",
        help=(
            "Conversion configuration: "
            "s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp"
        ),
    )
    parser_pdf.add_argument(
        "-p",
        "--punct",
        action="store_true",
        default=False,
        help="Enable punctuation conversion. (Default: False)",
    )
    parser_pdf.add_argument(
        "-H",
        "--header",
        action="store_true",
        default=False,
        help=(
            "Preserve page-break-like gaps when reflowing CJK paragraphs "
            "(passed as add_pdf_page_header to reflow_cjk_paragraphs)."
        ),
    )
    parser_pdf.add_argument(
        "-r",
        "--reflow",
        action="store_true",
        default=False,
        help="Enable CJK-aware paragraph reflow before conversion.",
    )
    parser_pdf.add_argument(
        "--compact",
        action="store_true",
        default=False,
        help="Use compact paragraph mode (single newline between paragraphs).",
    )
    parser_pdf.add_argument(
        "--timing",
        action="store_true",
        default=False,
        help="Show time use for each process workflow.",
    )
    parser_pdf.add_argument(
        "-e",
        "--extract",
        action="store_true",
        default=False,
        help="Extract PDF text only (skip OpenCC conversion).",
    )

    parser_pdf.set_defaults(func=subcommand_pdf)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
