from enum import Enum
from typing import Union, Optional, List, Tuple, TypedDict, Dict, Any, cast

from .opencc_pyo3 import (
    OpenCC as _OpenCC,
)


class OpenccConfig(str, Enum):
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
        """Parse a config string into an OpenccConfig enum value."""
        return cls(s.lower())


_ConfigLike = Union[str, OpenccConfig]
_CustomDictPair = Tuple[str, str]
_CustomDictSpecNative = List[Dict[str, Any]]


class CustomDictSpec(TypedDict, total=False):
    """
    In-memory custom dictionary specification for OpenCC.

    Fields:
        slot:
            Target OpenCC dictionary slot name such as
            "STPhrases", "TWPhrases", or "HKVariantsRevPhrases".

        pairs:
            List of (source, target) dictionary mappings.

        mode:
            Merge mode:
                - "append"   : Append entries to existing dictionary slot.
                - "override" : Replace the entire dictionary slot.
    """

    slot: str
    pairs: List[_CustomDictPair]
    mode: str


class CustomDictFileSpec(TypedDict, total=False):
    """
    File-based custom dictionary specification for OpenCC.

    Fields:
        slot:
            Target OpenCC dictionary slot name such as
            "STPhrases", "TWPhrases", or "HKVariantsRevPhrases".

        files:
            List of custom dictionary file paths.

        mode:
            Merge mode:
                - "append"   : Append entries to existing dictionary slot.
                - "override" : Replace the entire dictionary slot.
    """

    slot: str
    files: List[str]
    mode: str


class OpenCC(_OpenCC):
    CONFIG_LIST = [c.value for c in OpenccConfig]

    def __init__(self, config: _ConfigLike = "s2t"):
        # Native initialization is owned by the PyO3 backend.
        _ = config

    @classmethod
    def from_dicts(
            cls,
            config: _ConfigLike = "s2t",
            specs: Optional[List[CustomDictSpec]] = None,
    ) -> "OpenCC":
        """
        Create an OpenCC instance with in-memory custom dictionaries.

        Example spec:
            {
                "slot": "STPhrases",
                "pairs": [("帕兰蒂尔", "柏蘭蒂爾")],
                "mode": "append",
            }
        """
        cfg = cls._normalize_config(config)
        native_specs = cast(_CustomDictSpecNative, [] if specs is None else specs)
        return _OpenCC.from_dicts(cfg, native_specs)

    @classmethod
    def from_dict_files(
            cls,
            config: _ConfigLike = "s2t",
            specs: Optional[List[CustomDictFileSpec]] = None,
    ) -> "OpenCC":
        """
        Create an OpenCC instance with custom dictionary files.

        Example spec:
            {
                "slot": "STPhrases",
                "files": ["custom_st_phrases.txt"],
                "mode": "append",
            }
        """
        cfg = cls._normalize_config(config)
        native_specs = cast(_CustomDictSpecNative, [] if specs is None else specs)
        return _OpenCC.from_dict_files(cfg, native_specs)

    @staticmethod
    def _normalize_config(config: _ConfigLike) -> str:
        if isinstance(config, OpenccConfig):
            return config.value

        if isinstance(config, str):
            return config.lower()

        # Unknown type -> fallback safely
        return "s2t"

    def set_config(self, config):
        """
        Set the conversion configuration.
        :param config: One of OpenccConfig or a canonical string like "s2t".
        """
        cfg = self._normalize_config(config)
        self.apply_config(cfg)

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


__all__ = [
    "OpenCC",
    "OpenccConfig",
    "CustomDictSpec",
    "CustomDictFileSpec",
]
