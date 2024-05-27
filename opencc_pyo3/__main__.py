from __future__ import print_function

import argparse
import sys
import io
from opencc_pyo3 import OpenCC


def main():
    parser = argparse.ArgumentParser(formatter_class=argparse.ArgumentDefaultsHelpFormatter)
    parser.add_argument('-i', '--input', metavar='<file>',
                        help='Read original text from <file>.')
    parser.add_argument('-o', '--output', metavar='<file>',
                        help='Write converted text to <file>.')
    parser.add_argument('-c', '--config', metavar='<conversion>',
                        help='Conversion configuration: [s2t|s2tw|s2twp|s2hk|t2s|tw2s|tw2sp|hk2s|jp2t|t2jp]')
    parser.add_argument('-p', '--punct', action='store_true', default=False,
                        help='Punctuation conversion: True/False')
    parser.add_argument('--in-enc', metavar='<encoding>', default='UTF-8',
                        help='Encoding for input')
    parser.add_argument('--out-enc', metavar='<encoding>', default='UTF-8',
                        help='Encoding for output')
    args = parser.parse_args()

    if args.config is None:
        print("Please specify conversion.", file=sys.stderr)
        return 1

    opencc = OpenCC(args.config)

    if args.input:
        with io.open(args.input, encoding=args.in_enc) as f:
            input_str = f.read()
    else:
        input_str = sys.stdin.read()

    output_str = opencc.convert(input_str, args.punct)

    if args.output:
        with io.open(args.output, 'w', encoding=args.out_enc) as f:
            f.write(output_str)
    else:
        sys.stdout.write(output_str)
    in_from = args.input if args.input else "<stdin>"
    out_to = args.output if args.output else "stdout"
    print(f"Conversion completed ({args.config}): {in_from} -> {out_to}", file=sys.stderr)

    return 0


if __name__ == '__main__':
    sys.exit(main())
