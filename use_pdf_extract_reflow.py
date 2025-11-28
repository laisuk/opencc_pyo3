from opencc_pyo3 import extract_pdf_text, reflow_cjk_paragraphs

input_file = "tests/简体字.pdf"
# Note: This file will contain TEXT, not a real PDF. You may want ".txt" instead.
output_file = "tests/简体字_extracted.txt"


def main() -> None:
    # 1) Extract raw text from PDF
    print(f"Extracting text from: {input_file}")
    text = extract_pdf_text(input_file)

    if not text.strip():
        print("Warning: extracted text is empty or whitespace only.")
    else:
        print(f"Extracted {len(text)} characters from PDF.")

    # 2) Reflow CJK paragraphs
    #    - add_pdf_page_header = False → skip fake page gaps without end punctuation
    #    - compact = False → use blank line between paragraphs
    print("Reflowing CJK paragraphs...")
    reflowed = reflow_cjk_paragraphs(text, add_pdf_page_header=False, compact=False)

    # 3) Save result to disk (UTF-8)
    print(f"Writing reflowed text to: {output_file}")
    with open(output_file, "w", encoding="utf-8", newline="\n") as f:
        f.write(reflowed)

    print("Done.")


if __name__ == "__main__":
    main()
