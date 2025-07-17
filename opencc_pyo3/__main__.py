from __future__ import print_function

import argparse
import sys
import io
import os
from opencc_pyo3 import OpenCC
from .office_helper import OFFICE_FORMATS, convert_office_doc

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
            print("Input text to convert, <Ctrl+Z> (Windows) or <Ctrl+D> (Unix) then Enter to submit:", file=sys.stderr)

        input_str = sys.stdin.read()

    output_str = opencc.convert(input_str, args.punct)

    if args.output:
        with io.open(args.output, 'w', encoding=args.out_enc) as f:
            f.write(output_str)
    else:
        sys.stdout.write(output_str)
    in_from = args.input if args.input else "<stdin>"
    out_to = args.output if args.output else "stdout"
    if sys.stderr.isatty():
        print(f"Conversion completed ({args.config}): {in_from} -> {out_to}", file=sys.stderr)

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
        ext = f".{office_format}" if auto_ext and office_format and office_format in OFFICE_FORMATS else \
            input_ext
        output_file = os.path.join(input_dir, f"{input_name}_converted{ext}")
        print(f"ℹ️  Output file not specified. Using: {output_file}", file=sys.stderr)

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
            print(f"❌  Office document conversion failed: {message}", file=sys.stderr)
            return 1
    except Exception as ex:
        print(f"❌  Error during Office document conversion: {str(ex)}", file=sys.stderr)
        return 1

def main():
    parser = argparse.ArgumentParser(
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
        description="opencc_pyo3 an Rust-PyO3-based OpenCC CLI"
    )

    subparsers = parser.add_subparsers(dest='command', required=True)

    # Convert subcommand
    parser_convert = subparsers.add_parser('convert', help='Convert Chinese text using OpenCC')
    parser_convert.add_argument('-i', '--input', metavar='<file>',
                        help='Read original text from <file>.')
    parser_convert.add_argument('-o', '--output', metavar='<file>',
                        help='Write converted text to <file>.')
    parser_convert.add_argument('-c', '--config', metavar='<conversion>',
                        help='Conversion configuration: s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp')
    parser_convert.add_argument('-p', '--punct', action='store_true', default=False,
                        help='Enable punctuation conversion. (Default: False)')
    parser_convert.add_argument('--in-enc', metavar='<encoding>', default='UTF-8',
                        help='Encoding for input. (Default: UTF-8)')
    parser_convert.add_argument('--out-enc', metavar='<encoding>', default='UTF-8',
                        help='Encoding for output. (Default: UTF-8)')
    parser_convert.set_defaults(func=subcommand_convert)

    # Office subcommand
    parser_office = subparsers.add_parser('office', help='Convert Office document and Epub Chinese text using OpenCC')
    parser_office.add_argument('-i', '--input', metavar='<file>',
                               help='Input Office document from <file>.')
    parser_office.add_argument('-o', '--output', metavar='<file>',
                               help='Output Office document to <file>.')
    parser_office.add_argument('-c', '--config', metavar='<conversion>',
                               help='conversion: s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp')
    parser_office.add_argument('-p', '--punct', action='store_true', default=False,
                               help='Enable punctuation conversion. (Default: False)')
    parser_office.add_argument('-f', '--format', metavar='<format>',
                               help='Target Office format (e.g., docx, xlsx, pptx, odt, ods, odp, epub)')
    parser_office.add_argument('--auto-ext', action='store_true', default=False,
                               help='Auto-append extension to output file')
    parser_office.add_argument('--keep-font', action='store_true', default=False,
                               help='Preserve font-family information in Office content')
    parser_office.set_defaults(func=subcommand_office)

    args = parser.parse_args()
    return args.func(args)


if __name__ == '__main__':
    sys.exit(main())
