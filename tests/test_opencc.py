import unittest
from opencc_pyo3 import OpenCC

class TestOpenCC(unittest.TestCase):

    def test_valid_config(self):
        cc = OpenCC("t2s")
        self.assertEqual(cc.get_config(), "t2s")
        self.assertEqual(cc.get_last_error(), "")

    def test_invalid_config_fallback(self):
        cc = OpenCC("invalid")
        self.assertEqual(cc.get_config(), "s2t")
        self.assertIn("Invalid config", cc.get_last_error())

    def test_supported_configs(self):
        configs = OpenCC.supported_configs()
        self.assertIn("s2t", configs)
        self.assertIn("t2jp", configs)
        self.assertTrue(OpenCC.is_valid_config("t2s"))
        self.assertFalse(OpenCC.is_valid_config("abc"))

    def test_convert(self):
        cc = OpenCC("s2t")
        result = cc.convert("八千里路云和月")  # should convert to Traditional
        self.assertIsInstance(result, str)
        self.assertNotEqual(result, "")  # assuming conversion result is non-empty

    def test_zho_check(self):
        cc = OpenCC()
        simp_text = "八千里路云和月"
        trad_text = "八千里路雲和月"
        unknown = "hello world"
        self.assertEqual(cc.zho_check(simp_text), 2)
        self.assertEqual(cc.zho_check(trad_text), 1)
        self.assertEqual(cc.zho_check(unknown), 0)

    def test_apply_config(self):
        cc = OpenCC("t2s")
        cc.set_config("s2tw")
        self.assertEqual(cc.get_config(), "s2tw")
        cc.set_config("nonexistent")
        self.assertEqual(cc.get_config(), "s2t")
        self.assertIn("Invalid config", cc.get_last_error())


if __name__ == '__main__':
    unittest.main()
