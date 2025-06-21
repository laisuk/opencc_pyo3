from .opencc_pyo3 import OpenCC as _OpenCC


class OpenCC(_OpenCC):
    CONFIG_LIST = [
        "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s",
        "t2tw", "tw2t", "t2twp", "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t"
    ]
    def __init__(self, config="s2t"):
        self.config = config if config in self.CONFIG_LIST else "s2t"

    def set_config(self, config):
        """
        Set the conversion configuration.
        :param config: One of OpenCC.CONFIG_LIST
        """
        super().apply_config(config)

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
