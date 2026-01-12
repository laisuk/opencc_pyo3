from enum import Enum
from typing import Union

from .opencc_pyo3 import (
    OpenCC as _OpenCC,
    extract_pdf_text,
    reflow_cjk_paragraphs,
    extract_pdf_text_pages,
    extract_pdf_pages_with_callback
)
from .pdfium_helper import extract_pdf_pages_with_callback_pdfium


class OpenccConfig(Enum):
    S2T = "s2t"
    T2S = "t2s"
    S2TW = "s2tw"
    TW2S = "tw2s"
    S2TWP = "s2twp"
    TW2SP = "tw2sp"
    S2HK = "s2hk"
    HK2S = "hk2s"
    T2TW = "t2tw"
    TW2T = "tw2t"
    T2TWP = "t2twp"
    TW2TP = "tw2tp"
    T2HK = "t2hk"
    HK2T = "hk2t"
    T2JP = "t2jp"
    JP2T = "jp2t"

    value: str

    def to_canonical_name(self) -> str:
        """Return OpenCC canonical config name (e.g. 's2t')."""
        return self.value

    @classmethod
    def parse(cls, s: str) -> "OpenccConfig":
        return cls(s.lower())


_ConfigLike = Union[str, OpenccConfig]


class OpenCC(_OpenCC):
    CONFIG_LIST = [c.value for c in OpenccConfig]

    def __init__(self, config: _ConfigLike = "s2t"):
        # Normalize config to string, validate, then init native core
        cfg = self._normalize_config(config)
        self.config = cfg  # keep a Python-side mirror (optional but handy)
        # self.config = config if config in self.CONFIG_LIST else "s2t"

    @staticmethod
    def _normalize_config(config: _ConfigLike) -> str:
        if isinstance(config, OpenccConfig):
            return config.value

        if isinstance(config, str):
            c = config.lower()
            return c if c in OpenCC.CONFIG_LIST else "s2t"

        # Unknown type -> fallback safely
        return "s2t"

    def set_config(self, config):
        """
        Set the conversion configuration.
        :param config: One of OpenccConfig or a canonical string like "s2t".
        """
        cfg = self._normalize_config(config)
        super().apply_config(cfg)
        self.config = cfg

    def get_config(self):
        """
        Get the current conversion config.
        :return: Current config string
        """
        return super().get_config()

    @classmethod
    def supported_configs(cls):
        """
        Return a list of supported conversion config strings.
        :return: List of config names
        """
        return super().supported_configs()

    @classmethod
    def is_valid_config(cls, config):
        """
        Check validity of a conversion configuration string.
        :param config: Conversion configuration string
        :return: True if valid, False otherwise
        """
        return super().is_valid_config(config)

    def get_last_error(self):
        """
        Get the last error message from the underlying OpenCC core.
        :return: Error string or empty string if no error
        """
        return super().get_last_error()

    def zho_check(self, input_text):
        """
        Heuristically determine whether input text is Simplified or Traditional Chinese.
        :param input_text: Input string
        :return: 0 = unknown, 2 = simplified, 1 = traditional
        """
        return super().zho_check(input_text)

    def convert(self, input_text, punctuation=False):
        """
        Automatically dispatch to the appropriate conversion method based on `self.config.
        :param input_text: The string to convert
        :param punctuation: Whether to apply punctuation conversion
        :return: Converted string or error message
        """
        return super().convert(input_text, punctuation)
