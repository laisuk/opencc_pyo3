from .opencc_pyo3 import *


class OpenCC(opencc_pyo3.OpenCC):
    def __init__(self, config="s2t"):
        config_list = [
            "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s", "t2tw", "tw2t", "t2twp", "tw2t", "tw2tp",
            "t2hk", "hk2t", "t2jp", "jp2t"
        ]
        self.config = config if config in config_list else "s2t"

    def zho_check(self, input_text):
        return super().zho_check(input_text)

    def convert(self, input_text, punctuation=False):
        return super().convert(input_text, punctuation)
