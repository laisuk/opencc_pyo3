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
    """
    def __init__(self, config: str) -> None:
        """
        Initialize a new OpenCC instance.

        Args:
            config (str): Conversion config string.
        """
        self.config = config
        ...

    def convert(self, input_text: str, punctuation: bool) -> str:
        """
        Convert Chinese text using the current OpenCC config.

        Args:
            input_text (str): Input text.
            punctuation (bool): Whether to convert punctuation.

        Returns:
            str: Converted text.
        """
        ...

    def zho_check(self, input_text: str) -> int:
        """
        Detect the type of Chinese in the input text.

        Args:
            input_text (str): Input text.

        Returns:
            int: Integer code representing detected Chinese type.
            (1: Traditional, 2: Simplified, 0: Others)
        """
        ...

