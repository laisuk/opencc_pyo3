# use_pdfium_test.py
from opencc_pyo3.pdfium_helper import extract_pdf_pages_with_callback_pdfium

def on_page(page, total, text):
    print(f"[{page}/{total}] {len(text)} chars", end="\r", flush=True)

extract_pdf_pages_with_callback_pdfium("tests/简体字.pdf", on_page)

# Clear the progress line
print(" " * 80, end="\r")
print("Process completed")
