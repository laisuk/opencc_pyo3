# use_chuck_aba_reflow.py
from opencc_pyo3.opencc_pyo3 import reflow_cjk_paragraphs

input_file = "chunk_abc.txt"
output_file = "chunk_abc_reflowed.txt"


def main() -> None:
    # 1) Read raw text from input file (UTF-8)
    print(f"Reading text from: {input_file}")
    try:
        with open(input_file, "r", encoding="utf-8") as f:
            text = f.read()
    except FileNotFoundError:
        print(f"Error: input file not found: {input_file}")
        return

    if not text.strip():
        print("Warning: input text is empty or whitespace only.")
    else:
        print(f"Loaded {len(text)} characters from input file.")

    # 2) Reflow CJK paragraphs
    #    - add_pdf_page_header = False → skip fake page gaps without end punctuation
    #    - compact = False → use blank line between paragraphs
    print("Reflowing CJK paragraphs...")
    reflowed = reflow_cjk_paragraphs(text, add_pdf_page_header=False, compact=False)

    # 3) Save result to disk (UTF-8, Unix newlines)
    print(f"Writing reflowed text to: {output_file}")
    with open(output_file, "w", encoding="utf-8", newline="\n") as f:
        f.write(reflowed)

    print("Done.")


if __name__ == "__main__":
    main()
