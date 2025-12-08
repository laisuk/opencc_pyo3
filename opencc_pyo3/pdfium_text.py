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
#  Heuristics for detecting corrupted UTF-16 decode (Identity-H fallback)
# ==============================================================================

def _looks_broken_utf16(text: str) -> bool:
    """
    Detect whether the UTF-16 decoded string is actually UTF-8 bytes stuffed
    into 16-bit units. Identity-H fonts without proper ToUnicode CMaps often
    trigger this fallback behavior.
    """
    if not text:
        return False

    # Private-use area heuristic (commonly produced by mis-decoded UTF-16)
    bad = sum(1 for ch in text if 0xE000 <= ord(ch) <= 0xF8FF)

    # If 20%+ characters belong here, we assume PDFium handed us UTF-8 bytes.
    return bad > (len(text) * 0.20)


def _decode_pdfium_buffer(buf, extracted: int) -> str:
    """
    Perform C#-equivalent decoding of a PDFium UTF-16 buffer, with automatic
    fallback reconstruction of UTF-8 byte sequences when Identity-H fallback
    occurs inside PDFium.

    Returns a clean Unicode string with all '\x00' (PDFium segmentation marks)
    converted to '\n'.
    """
    # 1) Attempt normal UTF-16LE decode (correct for most PDFs)
    raw = ctypes.string_at(buf, extracted * 2)
    text16 = raw.decode("utf-16le", errors="ignore")

    # Good UTF-16 → done
    if not _looks_broken_utf16(text16):
        return text16.replace("\x00", "\n")

    # 2) Reconstruct UTF-8 byte stream from 16-bit units
    data = bytearray()
    for i in range(extracted):
        v = buf[i]
        # PDFium stores fallback bytes in high-byte-first order.
        data.append((v >> 8) & 0xFF)
        data.append(v & 0xFF)

    try:
        text8 = data.decode("utf-8")
    except (RuntimeError, Exception):
        text8 = data.decode("utf-8", errors="ignore")

    return text8.replace("\x00", "\n")


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
        text        : extracted Unicode text for the page

    Notes
    -----
    • Works for complex CJK fonts (Identity-H, CIDType0, missing ToUnicode).
    • Performs UTF-16 decode with UTF-8 fallback (same as PdfiumViewer).
    • Converts embedded NUL (U+0000) to newline for clean segmentation.
    • This function **does not** perform reflow; it only extracts raw text.
    """

    pdf_path_bytes = path.encode("utf-8")

    _pdfium.FPDF_InitLibrary()
    doc = _pdfium.FPDF_LoadDocument(pdf_path_bytes, None)

    if not doc:
        raise RuntimeError(f"PDFium failed to load document: {path}")

    try:
        total = _pdfium.FPDF_GetPageCount(doc)
        if total <= 0:
            callback(1, 1, "")
            return

        for i in range(total):
            page = _pdfium.FPDF_LoadPage(doc, i)
            # print("DEBUG PAGE HANDLE =", page)

            if not page:
                callback(i + 1, total, "")
                continue

            textpage = _pdfium.FPDFText_LoadPage(page)
            if not textpage:
                _pdfium.FPDF_ClosePage(page)
                callback(i + 1, total, "")
                continue

            # Count UTF-16 characters
            count = _pdfium.FPDFText_CountChars(textpage)

            if count > 0:
                buf = (ctypes.c_uint16 * (count + 1))()
                extracted = _pdfium.FPDFText_GetText(
                    textpage,
                    0, count,
                    buf,
                )

                if extracted > 0:
                    text = _decode_pdfium_buffer(buf, extracted)
                else:
                    text = ""
            else:
                text = ""

            # Cleanup
            _pdfium.FPDFText_ClosePage(textpage)
            _pdfium.FPDF_ClosePage(page)

            callback(i + 1, total, text)

    finally:
        _pdfium.FPDF_CloseDocument(doc)
        _pdfium.FPDF_DestroyLibrary()
