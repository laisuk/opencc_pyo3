"""
PDFium-based page-by-page text extraction for opencc_pyo3.

This module provides:
  • A stable ctypes binding for PDFium (stdcall on Windows).
  • C#-equivalent behavior for FPDFText_GetText().
  • Automatic UTF-16 / UTF-8 fallback decoding (handles Identity-H cases).
  • Automatic NUL → newline conversion (PDFium segmentation marker).
  • A clean callback-based interface for progress reporting.
"""

from __future__ import annotations

import ctypes
from typing import Callable, Any

from .pdfium_loader import load_pdfium

# ==============================================================================
#  PDFium basic types (equivalent to C# IntPtr)
# ==============================================================================
FPDF_DOCUMENT = ctypes.c_void_p
FPDF_PAGE = ctypes.c_void_p
FPDF_TEXTPAGE = ctypes.c_void_p

# ==============================================================================
#  Load PDFium (WinDLL on Windows, CDLL otherwise)
# ==============================================================================
_pdfium = load_pdfium()

# ==============================================================================
#  Bind PDFium function signatures (NO WINFUNCTYPE)
#  Equivalent to C# PdfiumNative signatures.
# ==============================================================================

# ---- Library init/close ----
_pdfium.FPDF_InitLibrary.argtypes = []
_pdfium.FPDF_InitLibrary.restype = None

_pdfium.FPDF_DestroyLibrary.argtypes = []
_pdfium.FPDF_DestroyLibrary.restype = None

# ---- Document ----
_pdfium.FPDF_LoadDocument.argtypes = [
    ctypes.c_char_p,  # UTF-8 filename
    ctypes.c_char_p,  # password (optional)
]
_pdfium.FPDF_LoadDocument.restype = FPDF_DOCUMENT

_pdfium.FPDF_CloseDocument.argtypes = [FPDF_DOCUMENT]
_pdfium.FPDF_CloseDocument.restype = None

# ---- Page operations ----
_pdfium.FPDF_GetPageCount.argtypes = [FPDF_DOCUMENT]
_pdfium.FPDF_GetPageCount.restype = ctypes.c_int

_pdfium.FPDF_LoadPage.argtypes = [FPDF_DOCUMENT, ctypes.c_int]
_pdfium.FPDF_LoadPage.restype = FPDF_PAGE

_pdfium.FPDF_ClosePage.argtypes = [FPDF_PAGE]
_pdfium.FPDF_ClosePage.restype = None

# ---- Text page ----
_pdfium.FPDFText_LoadPage.argtypes = [FPDF_PAGE]
_pdfium.FPDFText_LoadPage.restype = FPDF_TEXTPAGE

_pdfium.FPDFText_ClosePage.argtypes = [FPDF_TEXTPAGE]
_pdfium.FPDFText_ClosePage.restype = None

# ---- Text extraction ----
_pdfium.FPDFText_CountChars.argtypes = [FPDF_TEXTPAGE]
_pdfium.FPDFText_CountChars.restype = ctypes.c_int

_pdfium.FPDFText_GetText.argtypes = [
    FPDF_TEXTPAGE,
    ctypes.c_int,  # start index
    ctypes.c_int,  # count
    ctypes.POINTER(ctypes.c_uint16),  # UTF-16 buffer
]
_pdfium.FPDFText_GetText.restype = ctypes.c_int


# ==============================================================================
#  Compress multiple "\n" to Max = 2
# ==============================================================================
def _compress_newlines(text: str) -> str:
    """
    Reduce sequences of multiple newline characters to a maximum of two.
    This ensures:
      - Single '\n' = line break.
      - Double '\n\n' = paragraph boundary.
      - Prevents excessive blank-space inflation from Pdfium output.
    """
    out = []
    seen = 0

    for ch in text:
        if ch == "\n":
            seen += 1
            if seen <= 2:
                out.append("\n")
        else:
            seen = 0
            out.append(ch)

    return "".join(out)


def _decode_pdfium_buffer(buf: ctypes.Array, extracted: int) -> str:
    """
    Decode a UTF-16LE text buffer returned by Pdfium.

    Pdfium guarantees that:
      - All text is provided as UTF-16LE units.
      - The returned length (`extracted`) includes a trailing NUL terminator.
      - Pages with no textual content (e.g., image-only pages) often produce
        a buffer with a single NUL code unit.

    This decoder normalizes the output into clean Python strings:
      - Strips the trailing NUL (same behavior as C# implementation).
      - Returns a single '\n' to represent an empty/blank page, so that
        downstream reflow logic can treat it as an empty paragraph.
      - Normalizes CR/LF variations to LF.
      - Compresses runs of multiple newlines to at most two, preserving
        paragraph boundaries.
    """

    # No characters written → treat as no content.
    if extracted <= 0:
        return ""

    length = extracted

    # Strip trailing NUL, if present.
    if length > 0 and buf[length - 1] == 0:
        length -= 1

    # After removing the NUL, no text remains:
    # This page is empty (image-only or blank).
    # Represent it as a single '\n' so that the reflow layer
    # can recognize it as an empty paragraph.
    if length <= 0:
        return "\n"

    # Convert only the meaningful UTF-16 units to bytes.
    raw = ctypes.string_at(buf, length * 2)

    # Pdfium always produces valid UTF-16LE text.
    text = raw.decode("utf-16le", errors="ignore")

    # Normalize newline variants.
    text = text.replace("\r\n", "\n").replace("\r", "\n")

    # Compress sequences of 3+ newlines down to exactly 2,
    # ensuring consistent paragraph boundaries.
    text = _compress_newlines(text)

    return text


# ==============================================================================
#  Public Extraction API
# ==============================================================================

def extract_pdf_pages_with_callback_pdfium(
        path: str,
        callback: Callable[[int, int, str], Any],
        /,
):
    """
    Extract text from a PDF file page-by-page using PDFium,
    replicating the behavior of C# PdfiumNative.GetText().

    callback(page_number, total_pages, text)
        page_number : 1-based page index
        total_pages : total pages in PDF
        text        : extracted Unicode text for the page (page-break-safe)

    Notes
    -----
    • Works for complex CJK fonts (Identity-H, CIDType0, missing ToUnicode).
    • Performs UTF-16 decode with UTF-8 fallback (same as PdfiumViewer).
    • Converts embedded NUL (U+0000) to newline for clean segmentation.
    • This function **does not** perform reflow; it only extracts text.
    • IMPORTANT: Each callback `text` is guaranteed to end with a blank-line
      separator so page boundaries are never lost when concatenated.
    """

    def _normalize_page_text(s: str) -> str:
        # Normalize line endings first (PDFium sometimes yields odd combos)
        if s:
            s = s.replace("\r\n", "\n").replace("\r", "\n")

        # If blank/whitespace-only page, emit a visible blank separator
        if not s or s.strip() == "":
            return "\n"

        # Match the C# behavior: AppendLine(text.Trim()); AppendLine();
        # i.e. trimmed page text + a guaranteed blank line after it.
        s = s.strip()

        # Ensure page always ends with a blank line boundary
        # (two \n means there is at least one empty line after the last content line)
        return s + "\n\n"

    pdf_path_bytes = path.encode("utf-8")

    _pdfium.FPDF_InitLibrary()
    doc = _pdfium.FPDF_LoadDocument(pdf_path_bytes, None)

    if not doc:
        raise RuntimeError(f"PDFium failed to load document: {path}")

    try:
        total = _pdfium.FPDF_GetPageCount(doc)
        if total <= 0:
            callback(1, 1, "\n")
            return

        for i in range(total):
            page = _pdfium.FPDF_LoadPage(doc, i)
            if not page:
                callback(i + 1, total, "\n")
                continue

            textpage = _pdfium.FPDFText_LoadPage(page)
            if not textpage:
                _pdfium.FPDF_ClosePage(page)
                callback(i + 1, total, "\n")
                continue

            count = _pdfium.FPDFText_CountChars(textpage)

            if count > 0:
                buf = (ctypes.c_uint16 * (count + 1))()
                extracted = _pdfium.FPDFText_GetText(textpage, 0, count, buf)
                if extracted > 0:
                    raw = _decode_pdfium_buffer(buf, extracted)
                else:
                    raw = ""
            else:
                raw = ""

            _pdfium.FPDFText_ClosePage(textpage)
            _pdfium.FPDF_ClosePage(page)

            callback(i + 1, total, _normalize_page_text(raw))

    finally:
        _pdfium.FPDF_CloseDocument(doc)
        _pdfium.FPDF_DestroyLibrary()
