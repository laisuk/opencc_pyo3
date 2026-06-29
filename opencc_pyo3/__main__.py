from __future__ import print_function

import argparse
import sys

from opencc_pyo3 import OpenCC, OpenccConfig, CustomDictFileSpec

CONFIG_HELP = "Configuration: " + "|".join(OpenCC.supported_configs())


def resolve_config(config):
    if config is None:
        print("ℹ️  Config not set. Use default: s2t", file=sys.stderr)
        return "s2t"

    try:
        return OpenccConfig.parse(config).to_canonical_name()
    except ValueError:
        print(f"❌  Invalid OpenCC config: {config}", file=sys.stderr)
        print(
            f"   Supported configs: {' | '.join(OpenCC.supported_configs())}",
            file=sys.stderr,
        )
        return None


def parse_custom_dict_spec(spec: str) -> CustomDictFileSpec:
    parts = spec.split(":", 2)
    if len(parts) != 3:
        raise ValueError("Expected custom dictionary spec: slot:mode:path")

    slot, mode, path = (p.strip() for p in parts)

    if not slot:
        raise ValueError("Custom dictionary slot is empty.")
    if not mode:
        raise ValueError("Custom dictionary mode is empty.")
    if not path:
        raise ValueError("Custom dictionary path is empty.")

    result: CustomDictFileSpec = {
        "slot": slot,
        "mode": mode,
        "files": [path],
    }
    return result


def custom_dict_specs_from_args(args):
    return [parse_custom_dict_spec(s) for s in (getattr(args, "custom_dict", None) or [])]


def subcommand_convert(args):
    import io

    config = resolve_config(args.config)
    if config is None:
        return 1
    args.config = config

    # Plain text conversion fallback
    # opencc = OpenCC(config)
    try:
        specs = custom_dict_specs_from_args(args)
        opencc = OpenCC.from_dict_files(config, specs) if specs else OpenCC(config)
    except Exception as ex:
        print(f"❌  Invalid --custom-dict: {ex}", file=sys.stderr)
        return 1

    # Prompt user if input is from terminal
    if args.input is None and sys.stdin.isatty():
        print("Input text to convert, <Ctrl+Z>/<Ctrl+D> to submit:", file=sys.stderr)

    # Read input text (from file or stdin)
    with io.open(args.input if args.input else 0, encoding=args.in_enc) as f:
        input_str = f.read()

    # Optional pre-processing step: normalize CJK Compatibility Ideographs.
    if getattr(args, "norm_compat", False):
        input_str = opencc.normalize_compat(input_str)

    # Perform OpenCC conversion
    output_str = opencc.convert(input_str, args.punct)

    # Optional DeTofu display-safe fallback
    if args.detofu_file and not args.detofu:
        print("❌  --detofu-file requires --detofu", file=sys.stderr)
        return 1

    if args.detofu:
        level = args.detofu

        if args.detofu_file:
            output_str = opencc.detofu_with_custom_file(
                output_str,
                level,
                args.detofu_file,
            )
        else:
            output_str = opencc.detofu(output_str, level)

    # Write output text (to file or stdout)
    with io.open(args.output if args.output else 1, 'w', encoding=args.out_enc) as f:
        f.write(output_str)

    in_from = args.input if args.input else "<stdin>"
    out_to = args.output if args.output else "stdout"
    if sys.stderr.isatty():
        if not args.output and output_str and not output_str.endswith("\n"):
            print()
        # print(f"Conversion completed ({args.config}): {in_from} -> {out_to}", file=sys.stderr)
        status = f"Conversion completed ({args.config}"
        if args.detofu:
            status += f", detofu:{args.detofu}"
        status += f"): {in_from} -> {out_to}"
        print(status, file=sys.stderr)

    return 0


def subcommand_office(args):
    import os
    from pathlib import Path
    from .office_helper import OFFICE_FORMATS, convert_office_doc

    config = resolve_config(args.config)
    if config is None:
        return 1
    args.config = config

    input_file = args.input
    output_file = args.output
    office_format = args.format.lower() if args.format else None
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
    if not Path(input_file).is_file():
        print(f"❌ Input file not found: {input_file}", file=sys.stderr)
        return 1

    # Determine office format from file extension if not provided
    if office_format:
        if office_format not in OFFICE_FORMATS:
            print(f"❌  Unsupported Office format: {args.format}", file=sys.stderr)
            return 1
    else:
        file_ext = os.path.splitext(input_file)[1].lower().lstrip(".")
        if file_ext not in OFFICE_FORMATS:
            print(f"❌  Invalid Office file extension: .{file_ext or '(none)'}", file=sys.stderr)
            print("   Valid extensions: .docx | .xlsx | .pptx | .odt | .ods | .odp | .epub", file=sys.stderr)
            return 1
        office_format = str(file_ext)

    # If output file is not specified, generate one based on input file
    if not output_file:
        input_path = Path(input_file)
        input_dir = input_path.parent if input_path.parent != Path("") else Path.cwd()
        output_path = input_dir / f"{input_path.stem}_converted.{office_format}"
        output_file = str(output_path)
        print(f"ℹ️  Output file not specified. Using: {output_path}", file=sys.stderr)

    elif not os.path.splitext(output_file)[1]:
        output_file += f".{office_format}"
        print(f"ℹ️  Auto-extension applied: {output_file}", file=sys.stderr)

    try:
        specs = custom_dict_specs_from_args(args)
        opencc = OpenCC.from_dict_files(config, specs) if specs else OpenCC(config)
    except Exception as ex:
        print(f"❌  Invalid --custom-dict: {ex}", file=sys.stderr)
        return 1

    try:
        # Perform Office document conversion
        success, message = convert_office_doc(
            input_file,
            output_file,
            office_format,
            opencc,
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

    config = None if args.extract else resolve_config(args.config)
    if not args.extract and config is None:
        return 1
    if config is not None:
        args.config = config

    # ---------------------------------------------------------
    # PDFium extraction (single source of truth)
    # ---------------------------------------------------------
    pages: List[str] = []

    def _on_page(page: int, total: int, chunk: str) -> None:
        percent = page * 100 // total if total else 100
        msg = f"Loading [{page}/{total}] ({percent:3d}%) Extracted {len(chunk)} chars"
        width = 70  # safe for your 72-column Win7 console
        # Prevent wrapping
        msg = msg[:width]
        # Overwrite previous line
        print(msg.ljust(width), end="\r", flush=True)
        pages.append(chunk)

    def print_done(total: int) -> None:
        msg = f"Completed [{total}/{total}] (100%) ✓"
        width = 70
        if len(msg) < width:
            msg += " " * (width - len(msg))

        sys.stdout.write("\r" + msg + "\n")
        sys.stdout.flush()

    print(f"Extracting PDF page-by-page with PDFium: {p}")
    extract_pdf_pages_with_callback_pdfium(input_path_str, _on_page, args.header)
    # print()  # newline after progress
    print_done(len(pages))

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
        # opencc = OpenCC(str(config))
        config = str(config)
        try:
            specs = custom_dict_specs_from_args(args)
            opencc = OpenCC.from_dict_files(config, specs) if specs else OpenCC(config)
        except Exception as ex:
            print(
                f"⚠️  Invalid --custom-dict: {ex}\n"
                "   Skipping custom dictionaries and continuing with embedded dictionaries.",
                file=sys.stderr,
            )
            opencc = OpenCC(config)

        # Optional pre-processing step: normalize CJK Compatibility Ideographs.
        if getattr(args, "norm_compat", False):
            text = opencc.normalize_compat(text)
        text = opencc.convert(text, args.punct)
    else:
        if args.config or args.punct:
            print("ℹ️  --extract specified: skipping OpenCC conversion.", file=sys.stderr)

    # ---------------------------------------------------------
    # Write Output
    # ---------------------------------------------------------
    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)

    print(f"📄 Input : {p}")
    print(f"📁 Output: {output_path}")
    print("⚙️ Engine: pdfium")

    if args.extract:
        print("🧾 Mode  : extract-only (no OpenCC)")
    else:
        print(f"🧾 Config: {args.config} (punct: {'on' if args.punct else 'off'})")

    if args.reflow:
        options = []
        if args.compact:
            options.append("compact")
        if args.header:
            options.append("headers")

        suffix = f" ({', '.join(options)})" if options else ""
        print(f"📑 Reflow: on{suffix}")
    else:
        print("📑 Reflow: off")

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
        "convert", formatter_class=argparse.ArgumentDefaultsHelpFormatter, help="Convert Chinese text using OpenCC"
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
        help=CONFIG_HELP,
    )
    parser_convert.add_argument(
        "-p",
        "--punct",
        action="store_true",
        default=False,
        help="Enable punctuation conversion. (Default: False)",
    )
    parser_convert.add_argument(
        "-n",
        "--norm-compat",
        action="store_true",
        default=False,
        help="Normalize CJK Compatibility Ideographs before conversion. (Default: False)",
    )
    parser_convert.add_argument(
        "--detofu",
        nargs="?",
        const="all",
        default=None,
        metavar="<level>",
        help=(
            "Apply tofu-safe fallback after conversion. "
            "Levels: all/ExtB, ExtC, ExtD, ExtE, ExtF, ExtG, ExtH, ExtI."
        ),
    )
    parser_convert.add_argument(
        "--detofu-file",
        metavar="<file>",
        help=(
            "Load additional detofu fallback mappings from a UTF-8 text file. "
            "Custom mappings override built-in mappings; requires --detofu."
        ),
    )
    parser_convert.add_argument(
        "--custom-dict",
        action="append",
        metavar="<slot:mode:path>",
        help=(
            "Load custom dictionary file. "
            "Format: slot:mode:path, e.g. STPhrases:append:custom.txt. "
            "Can be used multiple times."
        ),
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
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
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
        help=CONFIG_HELP,
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
        "-k",
        "--keep-font",
        action="store_true",
        default=False,
        help="Preserve font-family information in Office content",
    )
    parser_office.add_argument(
        "--custom-dict",
        action="append",
        metavar="<slot:mode:path>",
        help=(
            "Load custom dictionary file. "
            "Format: slot:mode:path, e.g. STPhrases:append:custom.txt. "
            "Can be used multiple times."
        ),
    )

    parser_office.set_defaults(func=subcommand_office)

    # -------------
    # pdf subcommand
    # -------------
    parser_pdf = subparsers.add_parser(
        "pdf",
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
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
        help=CONFIG_HELP,
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
        "-C",
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
    parser_pdf.add_argument(
        "-n",
        "--norm-compat",
        action="store_true",
        default=False,
        help="Normalize CJK Compatibility Ideographs before conversion. (Default: False)",
    )
    parser_pdf.add_argument(
        "--custom-dict",
        action="append",
        metavar="<slot:mode:path>",
        help=(
            "Load custom dictionary file. "
            "Format: slot:mode:path, e.g. STPhrases:append:custom.txt. "
            "Can be used multiple times."
        ),
    )

    parser_pdf.set_defaults(func=subcommand_pdf)

    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
