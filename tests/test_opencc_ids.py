# tests/test_opencc_ids.py

import unittest

from opencc_pyo3 import OpenCC


class TestOpenCCIds(unittest.TestCase):
    def test_ids_default_converts_inside_ids(self) -> None:
        cc = OpenCC("t2s")

        self.assertEqual(cc.convert("⿰口馬"), "⿰口马")
        self.assertEqual(cc.convert("⿰氵漢"), "⿰氵汉")

    def test_ids_preserve_ids_keeps_inside_ids(self) -> None:
        cc = OpenCC("t2s", preserve_ids=True)

        self.assertEqual(cc.convert("⿰口馬"), "⿰口馬")
        self.assertEqual(cc.convert("⿰氵漢"), "⿰氵漢")

    def test_ids_preserve_ids_still_converts_outside_ids(self) -> None:
        cc = OpenCC("t2s", preserve_ids=True)

        self.assertEqual(
            cc.convert("漢字 ⿰氵漢 馬"),
            "汉字 ⿰氵漢 马",
        )

    def test_ids_default_false_keyword(self) -> None:
        cc = OpenCC("t2s", preserve_ids=False)

        self.assertEqual(
            cc.convert("漢字 ⿰氵漢 馬"),
            "汉字 ⿰氵汉 马",
        )


if __name__ == "__main__":
    unittest.main()