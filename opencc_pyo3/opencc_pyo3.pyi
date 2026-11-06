from typing import List, Optional, Tuple, TypedDict

_CustomDictPair = Tuple[str, str]


class _CustomDictSpecRequired(TypedDict):
    slot: str
    pairs: List[_CustomDictPair]


class CustomDictSpec(_CustomDictSpecRequired, total=False):
    mode: str


class _CustomDictFileSpecRequired(TypedDict):
    slot: str
    files: List[str]


class CustomDictFileSpec(_CustomDictFileSpecRequired, total=False):
    mode: str


class OpenCC:
    """Convert Chinese text using OpenCC dictionaries.

    Select a case-insensitive configuration code. An unknown code falls
    back to "s2t" and is reported by get_last_error(). Set preserve_ids=True
    to protect characters inside Ideographic Description Sequences.
    """

    config: str

    def __init__(
            self,
            config: Optional[str] = "s2t",
            preserve_ids: bool = False,
    ) -> None:
        """
        Initialize a new OpenCC instance.
        Args:
            config (str): Conversion config string.
            preserve_ids (bool): Preserve characters inside Unicode IDS structures.
        """
        ...

    @classmethod
    def from_dicts(cls, config: str = "s2t", specs: Optional[List[CustomDictSpec]] = None) -> "OpenCC":
        """Create a converter from embedded dictionaries and in-memory mappings.

        Each spec supplies a slot and a list of (source, target) pairs. The mode
        defaults to "append". Use "override" to replace the entire slot.
        Invalid slots or modes raise ValueError.
        """
        ...

    @classmethod
    def from_dict_files(cls, config: str = "s2t", specs: Optional[List[CustomDictFileSpec]] = None) -> "OpenCC":
        """Create a converter from embedded dictionaries and UTF-8 dictionary files.

        Each spec supplies a slot and a list of file paths. Each mapping uses
        source<TAB>target. The mode defaults to "append". Use "override" to replace
        the entire slot. Invalid slots, modes, or dictionary files raise ValueError.
        """
        ...

    def convert(self, input_text: str, punctuation: bool = False) -> str:
        """Convert input_text with the active configuration.

        Set punctuation=True to apply the configuration's punctuation mappings.
        """
        ...

    def zho_check(self, input_text: str) -> int:
        """Estimate the Chinese script used in input_text.

        Return 1 for Traditional Chinese, 2 for Simplified Chinese, or 0
        when the script is undetermined.
        """
        ...

    def get_config(self) -> str:
        """Return the active canonical configuration code."""
        ...

    def apply_config(self, config: str) -> None:
        """Set the active configuration from a case-insensitive code.

        An unknown code selects "s2t" and records a message in get_last_error().
        Applying a valid code clears that message.
        """
        ...

    @staticmethod
    def supported_configs() -> List[str]:
        """Return all supported canonical configuration codes."""
        ...

    @staticmethod
    def is_valid_config(config: str) -> bool:
        """Return whether the configuration code is supported."""
        ...

    @staticmethod
    def available_slots() -> List[str]:
        """Return canonical slot names for custom dictionary specifications."""
        ...

    def get_last_error(self) -> str:
        """Return the current configuration error message, or an empty string."""
        ...

    def normalize_compat(self, text: str) -> str:
        """Map CJK Compatibility Ideographs to their unified forms.

        For example, "金" becomes "金". Apply this optional pass before convert()
        when the input contains compatibility ideographs.
        """
        ...

    def normalize_compat_extended(self, text: str) -> str:
        """Apply extended compatibility mappings and normalize CJK ideographs.

        This optional pre-processing pass includes the mappings used by
        normalize_compat().
        """
        ...

    def detofu(self, text: str, level: str = "all") -> str:
        """Replace mapped CJK extension characters with display fallbacks.

        The level selects the first extension block to process. "all" starts
        at ExtB. ExtB through ExtI and their short forms B through I are
        case-insensitive and accept hyphens or underscores.
        An unsupported level raises ValueError.
        """
        ...

    def detofu_with_custom_file(
            self,
            text: str,
            level: str = "all",
            path: str = ...,
    ) -> str:
        """Apply DeTofu with custom mappings from a UTF-8 file.

        Each line contains a source character, a fallback character, and an
        extension label separated by tabs. Custom entries take precedence over
        built-in entries for the same source character. File errors raise OSError.
        A missing path or an unsupported level raises ValueError.
        """
        ...

    def detofu_with_custom_pairs(
            self,
            text: str,
            level: str = "all",
            pairs: List[Tuple[str, str]] = ...,
    ) -> str:
        """Apply DeTofu with custom (source, fallback) character pairs.

        Supply pairs as a list of tuples containing single-character strings.
        Custom entries take precedence over built-in entries for the same source
        character. Missing pairs or an unsupported level raises ValueError.
        """
        ...


def reflow_cjk_paragraphs(text: str, add_pdf_page_header: bool, compact: bool) -> str:
    """
    Reflow CJK paragraphs in PDF-extracted text.

    This function merges artificial line breaks while trying to preserve logical
    paragraphs, titles, and chapter headings. It is especially useful for text
    extracted from PDFs before passing it to OpenCC for conversion.

    Args:
        text: Raw text, such as text returned by PDFium helper extraction.
        add_pdf_page_header: If False, page-break-like blank lines that are not
            preceded by CJK punctuation may be skipped; if True, such gaps are kept.
        compact: If True, paragraphs are separated by a single newline;
            if False, paragraphs are separated by a blank line.

    Returns:
        Reflowed text with normalized CJK paragraphs.
    """
    ...
