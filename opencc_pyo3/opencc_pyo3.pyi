class OpenCC:
    def __init__(self, config: str) -> None:
        self.config = config
        ...

    def convert(self, input: str, punctuation: bool) -> str:
        ...

    def zho_check(self, input: str) -> int:
        ...
