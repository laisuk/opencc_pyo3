from enum import Enum
from typing import Union, Optional, List, Tuple, TypedDict, cast

from .opencc_pyo3 import (
    OpenCC as _OpenCC,
)


class OpenccConfig(str, Enum):
    """Canonical conversion configurations accepted by OpenCC."""

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
    T2HKP = "t2hkp"
    HK2T = "hk2t"
    HK2TP = "hk2tp"
    T2JP = "t2jp"
    JP2T = "jp2t"
    S2HKP = "s2hkp"
    HK2SP = "hk2sp"
    S2SEAL = "s2seal"
    T2SEAL = "t2seal"
    SEAL2S = "seal2s"
    SEAL2T = "seal2t"

    def to_canonical_name(self) -> str:
        """Return the lowercase configuration code, such as ``s2t``."""
        return self.value

    @classmethod
    def parse(cls, s: str) -> "OpenccConfig":
        """Parse a case-insensitive code or raise ValueError if it is unknown."""
        return cls(s.lower())


_ConfigLike = Union[str, OpenccConfig]
_CustomDictPair = Tuple[str, str]


class _CustomDictSpecRequired(TypedDict):
    slot: str
    pairs: List[_CustomDictPair]


class CustomDictSpec(_CustomDictSpecRequired, total=False):
    """In-memory mappings for one custom dictionary slot.

    Supply a canonical slot name such as "STPhrases" and a list of
    (source, target) pairs. The optional mode defaults to "append", which
    adds entries and replaces duplicate keys. Use "override" to replace
    the entire slot.
    """

    mode: str


class _CustomDictFileSpecRequired(TypedDict):
    slot: str
    files: List[str]


class CustomDictFileSpec(_CustomDictFileSpecRequired, total=False):
    """UTF-8 dictionary files for one custom dictionary slot.

    Supply a canonical slot name such as "STPhrases" and a list of file
    paths. Each mapping uses source<TAB>target. The optional mode defaults
    to "append", which adds entries and replaces duplicate keys. Use
    "override" to replace the entire slot.
    """

    mode: str


class OpenCC(_OpenCC):
    """Convert Chinese text with embedded OpenCC dictionaries.

    Select a case-insensitive config code or an OpenccConfig member. An
    unknown code falls back to "s2t" and is reported by get_last_error().
    Set preserve_ids=True to protect characters inside Ideographic
    Description Sequences during conversion.
    """

    CONFIG_LIST = [c.value for c in OpenccConfig]

    def __init__(
            self,
            config: _ConfigLike = "s2t",
            preserve_ids: bool = False,
    ) -> None:
        """Initialize the converter with a configuration and optional IDS protection.

        The default config is "s2t". Set preserve_ids=True to protect
        characters inside Ideographic Description Sequences.
        """
        # Native initialization is owned by the PyO3 backend.
        _ = config
        _ = preserve_ids

    @classmethod
    def from_dicts(
            cls,
            config: _ConfigLike = "s2t",
            specs: Optional[List[CustomDictSpec]] = None,
    ) -> "OpenCC":
        """Create a native converter with in-memory dictionary mappings.

        Pass a config code or OpenccConfig member and a list of CustomDictSpec
        mappings. With specs=None, use the embedded dictionaries alone.
        Invalid slots or merge modes raise ValueError.

        The result exposes the native API. Use config or apply_config()
        to change its active configuration.
        """
        cfg = cls._normalize_config(config)
        native_specs = [] if specs is None else specs
        return cast("OpenCC", _OpenCC.from_dicts(cfg, native_specs))

    @classmethod
    def from_dict_files(
            cls,
            config: _ConfigLike = "s2t",
            specs: Optional[List[CustomDictFileSpec]] = None,
    ) -> "OpenCC":
        """Create a native converter with UTF-8 custom dictionary files.

        Pass a config code or OpenccConfig member and a list of
        CustomDictFileSpec mappings. With specs=None, use the embedded
        dictionaries alone. Invalid slots, modes, or dictionary files
        raise ValueError.

        The result exposes the native API. Use config or apply_config()
        to change its active configuration.
        """
        cfg = cls._normalize_config(config)
        native_specs = [] if specs is None else specs
        return cast("OpenCC", _OpenCC.from_dict_files(cfg, native_specs))

    @staticmethod
    def _normalize_config(config: _ConfigLike) -> str:
        if isinstance(config, OpenccConfig):
            return config.value

        if isinstance(config, str):
            return config.lower()

        # Unknown type -> fallback safely
        return "s2t"

    def set_config(self, config: _ConfigLike) -> None:
        """Set the active configuration from a code or OpenccConfig member.

        An unknown code selects "s2t" and records a message in
        get_last_error(). Applying a valid code clears that message.
        """
        cfg = self._normalize_config(config)
        super().apply_config(cfg)

    def get_config(self) -> str:
        """Return the active canonical configuration code."""
        return super().get_config()

    @staticmethod
    def supported_configs() -> List[str]:
        """Return all supported canonical configuration codes."""
        return _OpenCC.supported_configs()

    @staticmethod
    def is_valid_config(config: _ConfigLike) -> bool:
        """Return whether the code or OpenccConfig member is supported."""
        return _OpenCC.is_valid_config(config)

    @staticmethod
    def available_slots() -> List[str]:
        """Return canonical slot names for custom dictionary specifications."""
        return _OpenCC.available_slots()

    def get_last_error(self) -> str:
        """Return the current configuration error message, or an empty string."""
        return super().get_last_error()

    def zho_check(self, input_text: str) -> int:
        """Estimate the Chinese script used in input_text.

        Return 1 for Traditional Chinese, 2 for Simplified Chinese, or
        0 when the script is undetermined.
        """
        return super().zho_check(input_text)

    def convert(self, input_text: str, punctuation: bool = False) -> str:
        """Convert input_text with the active configuration.

        Set punctuation=True to apply the configuration's punctuation mappings.
        """
        return super().convert(input_text, punctuation)

    def normalize_compat(self, text: str) -> str:
        """Map CJK Compatibility Ideographs to their unified forms.

        For example, "金" becomes "金". Apply this optional pass before
        convert() when the input contains compatibility ideographs.
        """
        return super().normalize_compat(text)

    def normalize_compat_extended(self, text: str) -> str:
        """Apply extended compatibility mappings and normalize CJK ideographs.

        This optional pre-processing pass includes the mappings used by
        normalize_compat().
        """
        return super().normalize_compat_extended(text)

    def detofu(self, text: str, level: str = "all") -> str:
        """Replace mapped CJK extension characters with display fallbacks.

        The level selects the first extension block to process. "all"
        starts at ExtB. ExtB through ExtI and their short forms B through I
        are case-insensitive and accept hyphens or underscores.
        An unsupported level raises ValueError.
        """
        return super().detofu(text, level)

    def detofu_with_custom_file(
            self,
            text: str,
            level: str = "all",
            path: Optional[str] = None,
    ) -> str:
        """Apply DeTofu with custom mappings from a UTF-8 file.

        Supply path, for example detofu_with_custom_file(text, path="map.txt").
        Each line contains a source character, a fallback character, and an
        extension label separated by tabs. Custom entries take precedence
        over built-in entries for the same source character.

        A missing path raises TypeError. File errors raise OSError and an
        unsupported level raises ValueError.
        """
        if path is None:
            raise TypeError("detofu_with_custom_file() missing required argument: 'path'")

        return super().detofu_with_custom_file(text, level, path)

    def detofu_with_custom_pairs(
            self,
            text: str,
            level: str = "all",
            pairs: Optional[List[Tuple[str, str]]] = None,
    ) -> str:
        """Apply DeTofu with custom (source, fallback) character pairs.

        Supply pairs as a list of tuples containing single-character strings.
        Custom entries take precedence over built-in entries for the same
        source character. A missing pairs argument raises TypeError and an
        unsupported level raises ValueError.
        """
        if pairs is None:
            raise TypeError("detofu_with_custom_pairs() missing required argument: 'pairs'")

        return super().detofu_with_custom_pairs(text, level, pairs)


__all__ = [
    "OpenCC",
    "OpenccConfig",
    "CustomDictSpec",
    "CustomDictFileSpec",
]
