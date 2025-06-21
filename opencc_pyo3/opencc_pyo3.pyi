from typing import List


class OpenCC:
    """
    Python binding for OpenCC and Jieba functionalities.

    Provides Chinese text conversion (Simplified/Traditional), segmentation, and keyword extraction.

    Args:
        config (str): Optional conversion config (default: "s2t"). Must be one of:
            "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s",
            "t2tw", "tw2t", "t2twp", "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t".

    Attributes:
        config (str): Current OpenCC config string.
        last_error (str): Last error message, if any.
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
