class OpenCC:
    def __init__(self, config: str) -> None:
        self.config = config
        ...

    def convert(self, input_text: str, punctuation: bool) -> str:
        ...

    def zho_check(self, input_text: str) -> int:
        ...

