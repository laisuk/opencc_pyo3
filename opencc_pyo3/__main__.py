from __future__ import print_function

import argparse
import sys
import io
import os
import time
from pathlib import Path
from typing import List

from opencc_pyo3 import OpenCC, extract_pdf_text, reflow_cjk_paragraphs
from .office_helper import OFFICE_FORMATS, convert_office_doc

try:
    # If there's PDFium backend，can import； otherwise fallback
    from . import extract_pdf_pages_with_callback_pdfium
except (RuntimeError, Exception):  # ImportError 或 RuntimeError 由 lazy loader 拋出都算
    extract_pdf_pages_with_callback_pdfium = None  # type: ignore[assignment]


def subcommand_convert(args):
    if args.config is None:
        print("ℹ️  Config not specified. Using default: s2t", file=sys.stderr)
        args.config = "s2t"

    opencc = OpenCC(args.config)

    if args.input:
        with io.open(args.input, encoding=args.in_enc) as f:
            input_str = f.read()
    else:
        # Prompt only if reading from stdin, and it's interactive (i.e., not piped or redirected)
        if args.input is None and sys.stdin.isatty():
            print(
                "Input text to convert, <Ctrl+Z> (Windows) or <Ctrl+D> (Unix) then Enter to submit:",
                file=sys.stderr,
            )

        input_str = sys.stdin.read()

    output_str = opencc.convert(input_str, args.punct)

    if args.output:
        with io.open(args.output, "w", encoding=args.out_enc) as f:
            f.write(output_str)
    else:
        sys.stdout.write(output_str)

    in_from = args.input if args.input else "<stdin>"
    out_to = args.output if args.output else "stdout"
    if sys.stderr.isatty():
        print(
            f"Conversion completed ({args.config}): {in_from} -> {out_to}",
            file=sys.stderr,
        )

    return 0


def subcommand_office(args):
    input_file = args.input
    output_file = args.output
    office_format = args.format
    auto_ext = getattr(args, "auto_ext", False)
    config = args.config
    punct = args.punct
    keep_font = getattr(args, "keep_font", False)

    if args.config is None:
        print("ℹ️  Config not specified. Using default: s2t", file=sys.stderr)
        args.config = "s2t"
        config = args.config

    # Check for missing input/output files
    if not input_file and not output_file:
        print("❌  Input and output files are missing.", file=sys.stderr)
        return 1
    if not input_file:
        print("❌  Input file is missing.", file=sys.stderr)
        return 1

    # If output file is not specified, generate one based on input file
    if not output_file:
        input_name = os.path.splitext(os.path.basename(input_file))[0]
        input_ext = os.path.splitext(os.path.basename(input_file))[1]
        input_dir = os.path.dirname(input_file) or os.getcwd()
        ext = (
            f".{office_format}"
            if auto_ext and office_format and office_format in OFFICE_FORMATS
            else input_ext
        )
        output_file = os.path.join(input_dir, f"{input_name}_converted{ext}")
        print(f"ℹ️  Output file not specified. Using: {output_file}", file=sys.stderr)

    # Determine office format from file extension if not provided
    if not office_format:
        file_ext = os.path.splitext(input_file)[1].lower()
        if file_ext[1:] not in OFFICE_FORMATS:
            print(f"❌  Invalid Office file extension: {file_ext}", file=sys.stderr)
            print(
                "   Valid extensions: .docx | .xlsx | .pptx | .odt | .ods | .odp | .epub",
                file=sys.stderr,
            )
            return 1
        office_format = file_ext[1:]

    # Auto-append extension to output file if needed
    if (
            auto_ext
            and output_file
            and not os.path.splitext(output_file)[1]
            and office_format in OFFICE_FORMATS
    ):
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
            print(
                f"{message}\n📁  Output saved to: {os.path.abspath(output_file)}",
                file=sys.stderr,
            )
            return 0
        else:
            print(
                f"❌  Office document conversion failed: {message}", file=sys.stderr
            )
            return 1
    except Exception as ex:
        print(
            f"❌  Error during Office document conversion: {str(ex)}", file=sys.stderr
        )
        return 1


def subcommand_pdf(args) -> int:
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

    engine = getattr(args, "engine", "auto")

    if args.timing:
        t0_total = time.perf_counter()

    # text: str = ""

    # ---------------------------------------------------------
    # AUTO ENGINE SELECTION
    # ---------------------------------------------------------
    if engine == "auto":
        pdfium_available = extract_pdf_pages_with_callback_pdfium is not None

        if pdfium_available:
            try:
                pages: List[str] = []

                def _on_page(page: int, total: int, chunk: str) -> None:
                    msg = f"[{page}/{total}] Extracted {len(chunk)} chars"
                    print(msg.ljust(80), end="\r", flush=True)
                    pages.append(chunk)

                extract_pdf_pages_with_callback_pdfium(input_path_str, _on_page)
                print()
                text = "".join(pages)

                engine_used = "pdfium"

            except Exception as exc:
                print(
                    f"⚠️  PDFium extraction failed ({exc}). "
                    "Falling back to pure-Rust extractor.",
                    file=sys.stderr,
                )
                print("Extracting PDF text...please wait...")
                text = extract_pdf_text(input_path_str)
                engine_used = "rust"
        else:
            print("⚠️  PDFium backend is not available; using pure-Rust extractor.")
            print("Extracting PDF text...please wait...")
            text = extract_pdf_text(input_path_str)
            engine_used = "rust"

    # ---------------------------------------------------------
    # FORCE PDFIUM ENGINE
    # ---------------------------------------------------------
    elif engine == "pdfium":
        if extract_pdf_pages_with_callback_pdfium is None:
            print(
                "⚠️  PDFium backend not available. Falling back to Rust.",
                file=sys.stderr,
            )
            print("Extracting PDF text...please wait...")
            text = extract_pdf_text(input_path_str)
            engine_used = "rust"
        else:
            try:
                pages: List[str] = []

                def _on_page(page: int, total: int, chunk: str) -> None:
                    msg = f"[{page}/{total}] Extracted {len(chunk)} chars"
                    print(msg.ljust(80), end="\r", flush=True)
                    pages.append(chunk)

                extract_pdf_pages_with_callback_pdfium(input_path_str, _on_page)
                print()
                text = "".join(pages)
                engine_used = "pdfium"

            except Exception as exc:
                print(
                    f"⚠️  PDFium extraction failed ({exc}). "
                    "Falling back to Rust extractor.",
                    file=sys.stderr,
                )
                print("Extracting PDF text...please wait...")
                text = extract_pdf_text(input_path_str)
                engine_used = "rust"

    # ---------------------------------------------------------
    # FORCE RUST ENGINE
    # ---------------------------------------------------------
    else:  # engine == "rust"
        print("Extracting PDF text...please wait...")
        text = extract_pdf_text(input_path_str)
        engine_used = "rust"

    # Timing
    if args.timing:
        t1_extract = time.perf_counter()
        print(f"[timing] PDF extract: {(t1_extract - t0_total) * 1000:.1f} ms")

    # ---------------------------------------------------------
    # Reflow (optional)
    # ---------------------------------------------------------
    if args.reflow:
        text = reflow_cjk_paragraphs(
            text,
            add_pdf_page_header=args.header,
            compact=args.compact,
        )

    # ---------------------------------------------------------
    # OpenCC Conversion (optional)
    # ---------------------------------------------------------
    # --extract means: extraction only, no OpenCC conversion.
    if not args.extract:
        if args.config:
            opencc = OpenCC(args.config)
            text = opencc.convert(text, args.punct)
    else:
        # Optional: warn if user provided config/punct but asked extract-only
        if args.config or args.punct:
            print("ℹ️  --extract specified: skipping OpenCC conversion.", file=sys.stderr)

    # ---------------------------------------------------------
    # Write Output
    # ---------------------------------------------------------
    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)

    print(f"📄 Input : {input_path}")
    print(f"📁 Output: {output_path}")
    print(f"⚙️ Engine used: {engine_used}")
    if args.extract:
        print("🧾 Mode : extract-only (no OpenCC)")
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
        "--engine",
        metavar="<engine>",
        choices=["auto", "rust", "pdfium"],
        default="auto",
        help=(
            "PDF extraction engine:\n"
            "  auto   – Prefer PDFium; fallback to Rust if unavailable (recommended)\n"
            "  rust   – Pure-Rust extractor (no page progress)\n"
            "  pdfium – PDFium backend with per-page progress; fallback if it fails\n"
            "(Default: auto)"
        ),
    )
    parser_pdf.add_argument(
        "-E",
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
