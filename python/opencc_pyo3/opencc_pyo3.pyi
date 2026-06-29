from typing import List, Optional, Dict, Any, Tuple


class OpenCC:
    """
    Python binding for OpenCC text conversion.

    Provides Chinese text conversion between Simplified, Traditional,
    Hong Kong, Taiwan, and Japanese Kanji variants using OpenCC-compatible
    configurations.

    Args:
        config (str): Optional conversion config (default: "s2t"). Must be one of:
            "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s",
            "s2hkp", "hk2sp", "t2tw", "tw2t", "t2twp", "tw2tp", "t2hk",
            "hk2t", "t2jp", "jp2t".

    Attributes:
        config (str): Current OpenCC config string.
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
    def from_dicts(cls, config: str = "s2t", specs: Optional[List[Dict[str, Any]]] = None) -> "OpenCC":
        """
        Create an OpenCC instance with in-memory custom dictionaries.
        """
        ...

    @classmethod
    def from_dict_files(cls, config: str = "s2t", specs: Optional[List[Dict[str, Any]]] = None) -> "OpenCC":
        """
        Create an OpenCC instance with custom dictionary files.
        """
        ...

    def convert(self, input_text: str, punctuation: bool = False) -> str:
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

    @staticmethod
    def supported_configs() -> List[str]:
        """
        Get the supported Config list.
        :return: List of supported config strings.
        """
        ...

    @staticmethod
    def is_valid_config(config: str) -> bool:
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

    def normalize_compat(self, text: str) -> str:
        """
        Normalize CJK Compatibility Ideographs using the built-in Unicode table.

        This is an optional Unicode compatibility normalization pre-pass. It does
        not modify this OpenCC instance, its selected config, conversion
        dictionaries, segmentation behavior, script detection, or punctuation
        conversion.

        Use this before ``convert()`` when input may contain CJK Compatibility
        Ideographs such as ``金``. Unmapped compatibility ideographs remain
        unchanged.

        DeToFu is the opposite side of the pipeline: compatibility ideograph
        normalization is a pre-processing step, while ``detofu()`` is an
        optional post-processing display fallback.

        :param text: Input text.
        :return: Normalized text.
        """
        ...

    def detofu(self, text: str, level: str = "all") -> str:
        """
        Convert non-BMP CJK extension characters to display-safe fallbacks.

        This is a display compatibility pass. It does not affect OpenCC
        conversion dictionaries, phrase matching, regional variants, script
        detection, or punctuation conversion.

        :param text: Input text.
        :param level: CJK extension threshold: "all", "ExtB", "ExtC", ..., "ExtI".
                      Compact forms "B"..."I" are also accepted.
                      "all"/"ExtB" replaces ExtB and above; "ExtI" replaces ExtI only.
        :return: Display-safe text.
        """
        ...

    def detofu_with_custom_file(
            self,
            text: str,
            level: str = "all",
            path: str = ...,
    ) -> str:
        """
        Convert non-BMP CJK extension characters using built-in detofu mappings
        plus a custom UTF-8 fallback file.

        File format:
            tofu_char<TAB>fallback_char<TAB>extension

        Example:
            𣭲    氄    B
        """
        ...

    def detofu_with_custom_pairs(
            self,
            text: str,
            level: str = "all",
            pairs: List[Tuple[str, str]] = ...,
    ) -> str:
        """
        Convert non-BMP CJK extension characters using built-in detofu mappings
        plus custom fallback character pairs.

        Example:
            cc.detofu_with_custom_pairs("𣭲毛", "all", [("𣭲", "氄")])
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
