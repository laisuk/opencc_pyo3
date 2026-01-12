from typing import Any, Callable, List

class OpenCC:
    """
    Python binding for OpenCC and Jieba functionalities.

    Provides Chinese text conversion (Simplified/Traditional), segmentation, and keyword extraction.

    Args:
        config (str): Optional conversion config (default: "s2t"). Must be one of:
            "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s",
            "t2tw", "tw2t", "t2twp", "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t".

    Attributes:
        self.config (str): Current OpenCC config string.
        self.last_error (str): Last error message, if any.
    """

    def __init__(self, config: str) -> None:
        """
        Initialize a new OpenCC instance.
        Args:
            config (str): Conversion config string.
        """
        self.config: str
        self.last_error: str
        ...

    def convert(self, input_text: str, punctuation: bool) -> str:
        """
        Convert Chinese text using the current OpenCC config.
        :param input_text: Input text.
        :param punctuation: Whether to convert punctuation.
        :return str: Converted text.
        """
        ...

    def zho_check(self, input_text: str) -> int:
        """
        Detect the type of Chinese in the input text.
        :param input_text: Input text.
        :return int: Integer code representing detected Chinese type.
                (1: Traditional, 2: Simplified, 0: Others)
        """
        ...

    def get_config(self) -> str:
        """
        Get the current conversion config.
        :return: Current config string
        """
        ...

    def apply_config(self, config: str) -> None:
        """
        Set current config, reverts to "s2t" if invalid config value provided.
        :param config: Config string to be changed.
        """
        ...

    def supported_configs(self) -> List[str]:
        """
        Get the supported Config list.
        :return: List of supported config strings.
        """
        ...

    def is_valid_config(self, config: str) -> bool:
        """
        Check validity of the config string.
        :param config: Config string to be checked.
        """
        ...

    def get_last_error(self) -> str:
        """
        Get the last error message from the converter.
        :return str: Error message, or an empty string if no error occurred.
        """
        ...


def extract_pdf_text(path: str) -> str:
    """
    Extract plain text from a PDF file.

    This uses the Rust backend (pdf-extract or other implementation) to read the
    PDF at the given path and return its textual content as a single string.

    Args:
        path: Path to the PDF file on disk.

    Returns:
        Concatenated text of all pages.
    """
    ...


def reflow_cjk_paragraphs(text: str, add_pdf_page_header: bool, compact: bool) -> str:
    """
    Reflow CJK paragraphs in PDF-extracted text.

    This function merges artificial line breaks while trying to preserve logical
    paragraphs, titles, and chapter headings. It is especially useful for text
    extracted from PDFs before passing it to OpenCC for conversion.

    Args:
        text: Raw text (typically from ``extract_pdf_text``).
        add_pdf_page_header: If False, page-break-like blank lines that are not
            preceded by CJK punctuation may be skipped; if True, such gaps are kept.
        compact: If True, paragraphs are separated by a single newline;
            if False, paragraphs are separated by a blank line.

    Returns:
        Reflowed text with normalized CJK paragraphs.
    """
    ...


def extract_pdf_text_pages(path: str, /) -> List[str]:
    """
    Extracts plain text from a PDF file, split by pages.

    This uses the pure-Rust `pdf-extract` backend. It returns one string per
    page in reading order. This is useful when you want to process pages
    individually or show a progress bar while iterating.
    """
    ...


def extract_pdf_pages_with_callback(
        path: str,
        callback: Callable[[int, int, str], Any],
        /,
) -> None:
    """
    Incrementally extracts text from each page of a PDF and invokes a Python
    callback for each page as:

        callback(page_number, total_pages, text)

    Parameters
    ----------
    path : str
        Path to the PDF file on disk.
    callback : Callable[[int, int, str], Any]
        A function receiving (page_number, total_pages, text) for each page.

    == Behavior ==
    --------
    • If the PDF has a valid page tree (most standard PDFs, especially
      Word-exported PDFs), pages are extracted one-by-one and streamed to the
      callback.

    • If a page contains no text or extraction fails for a single page, that
      page is returned as an empty string "" and processing continues.

    • If *pdf-extract* cannot understand the PDF at all (common for complex
      commercial e-books or PDFs requiring advanced CMap/ToUnicode handling),
      a RuntimeError is raised. This is normal: these PDFs require a
      PDFium-based backend.

    Raises
    ------
    RuntimeError
        If the pure-Rust PDF extractor (`pdf-extract`) cannot parse the PDF
        structure or extract any text at all. This often happens with:
            • commercial/DRM-like or publisher-grade novel PDFs
            • PDFs without a standard /Pages tree
            • PDFs with compressed object streams or complex CMap encodings

        In these cases, use a PDFium-based extraction engine instead.

    Notes
    -----
    This function does not use PDFium. It is intended as a lightweight,
    pure-Rust fallback. For the highest compatibility with real-world
    East Asian text PDFs (HK/TW/CN/KR/JP novels, EPUB-converted PDFs,
    commercial publishers), a PDFium backend is recommended.
    """
    ...