import os
import tempfile

import sys
import unittest
from unittest import mock
import zipfile
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace

from typing import List, Optional

ROOT_DIR = Path(__file__).resolve().parents[1]
PYTHON_SRC_DIR = ROOT_DIR / "python"
if str(PYTHON_SRC_DIR) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC_DIR))

from opencc_pyo3 import OpenCC, CustomDictSpec, CustomDictFileSpec
from opencc_pyo3.__main__ import subcommand_convert
from opencc_pyo3 import pdfium_loader
from opencc_pyo3.office_helper import convert_office_doc


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
        result = cc.convert("八千里路云和月")
        self.assertIsInstance(result, str)
        self.assertNotEqual(result, "")

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
        self.assertEqual(cc.get_last_error(), "")

        cc.set_config("nonexistent")
        self.assertEqual(cc.get_config(), "s2t")
        self.assertIn("Invalid config", cc.get_last_error())

        cc.apply_config("nonexistent")
        self.assertEqual(cc.get_config(), "s2t")
        self.assertIn("Invalid config", cc.get_last_error())

    def test_convert_office_doc_generates_output_when_none(self):
        converter = OpenCC("s2t")

        with TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            input_path = tmp_path / "sample.docx"

            with zipfile.ZipFile(input_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                archive.writestr(
                    "word/document.xml",
                    '<?xml version="1.0" encoding="UTF-8"?><w:document><w:t>简体</w:t></w:document>',
                )

            success, message = convert_office_doc(
                str(input_path),
                None,
                "docx",
                converter,
            )

            output_path = tmp_path / "sample_converted.docx"
            self.assertTrue(success, message)
            self.assertTrue(output_path.is_file())

            with zipfile.ZipFile(output_path, "r") as archive:
                content = archive.read("word/document.xml").decode("utf-8")
                self.assertIn("簡體", content)

    def test_pdfium_loader_prefers_current_pyinstaller_layout(self):
        with TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            (tmp_path / "opencc_pyo3" / "pdfium").mkdir(parents=True)
            (tmp_path / "pdfium").mkdir(parents=True)

            with mock.patch.object(pdfium_loader.sys, "frozen", True, create=True), mock.patch.object(pdfium_loader.sys,
                                                                                                      "_MEIPASS",
                                                                                                      str(tmp_path),
                                                                                                      create=True), mock.patch.object(
                pdfium_loader, "__file__", str(tmp_path / "somewhere" / "pdfium_loader.py")):
                self.assertEqual(pdfium_loader._module_dir(), tmp_path / "opencc_pyo3")

    def test_pdfium_loader_falls_back_to_module_dir_when_needed(self):
        with TemporaryDirectory() as tmpdir:
            tmp_path = Path(tmpdir)
            module_root = tmp_path / "custom_pkg"
            (module_root / "pdfium").mkdir(parents=True)

            with mock.patch.object(pdfium_loader.sys, "frozen", True, create=True), mock.patch.object(pdfium_loader.sys,
                                                                                                      "_MEIPASS",
                                                                                                      str(tmp_path / "missing_root"),
                                                                                                      create=True), mock.patch.object(
                pdfium_loader, "__file__", str(module_root / "pdfium_loader.py")):
                self.assertEqual(pdfium_loader._module_dir(), module_root)

    # Test Custom Dicts
    def test_from_dicts_custom_st_phrases_palantir(self):
        specs: List[CustomDictSpec] = [
            {
                "slot": "STPhrases",
                "pairs": [("帕兰蒂尔", "柏蘭蒂爾")],
                "mode": "append",
            }
        ]

        cc = OpenCC.from_dicts("s2t", specs)

        self.assertEqual(cc.get_config(), "s2t")
        self.assertEqual(cc.convert("帕兰蒂尔是一家公司"), "柏蘭蒂爾是一家公司")

    def test_from_dict_files_custom_st_phrases_palantir(self):
        with TemporaryDirectory() as tmpdir:
            dict_path = Path(tmpdir) / "custom_st_phrases.txt"
            dict_path.write_text("帕兰蒂尔\t柏蘭蒂爾\n", encoding="utf-8")

            specs: List[CustomDictFileSpec] = [
                {
                    "slot": "STPhrases",
                    "files": [str(dict_path)],
                    "mode": "append",
                }
            ]

            cc = OpenCC.from_dict_files("s2t", specs)

            self.assertEqual(cc.get_config(), "s2t")
            self.assertEqual(cc.convert("帕兰蒂尔是一家公司"), "柏蘭蒂爾是一家公司")

    def test_opencc_detofu(self):
        cc = OpenCC()
        input_text = "𠉂𪠟𫝈𫬐"

        assert cc.detofu(input_text, "ExtE") == "𠉂𪠟𫝈㘔"
        assert cc.detofu(input_text, "ExtB") == "㒓㓄㑮㘔"
        assert cc.detofu(input_text, "all") == "㒓㓄㑮㘔"
        assert cc.detofu(input_text, "ext-c") == "𠉂㓄㑮㘔"

    def test_opencc_t2s_detofu_default_level(self):
        cc = OpenCC("t2s")

        output = cc.detofu(
            cc.convert("儼驂騑於上路，訪風景於崇阿")
        )

        assert output == "俨骖騑于上路，访风景于崇阿"

    def test_detofu_with_custom_file(self):
        with tempfile.NamedTemporaryFile(
                mode="w",
                encoding="utf-8",
                suffix=".txt",
                delete=False,
        ) as f:
            f.write("𣭲\t氄\tB\n")
            temp_path = f.name

        try:
            cc = OpenCC()

            result = cc.detofu_with_custom_file(
                "𣭲毛",
                "all",
                temp_path,
            )

            assert result == "氄毛"

        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)

    def test_opencc_detofu_with_custom_pairs(self):
        cc = OpenCC()

        output = cc.detofu_with_custom_pairs(
            "𣭲毛 骖𬴂",
            "all",
            [
                ("𣭲", "氄"),
                ("𬴂", "騑"),
            ],
        )

        assert output == "氄毛 骖騑"

    def test_opencc_detofu_custom_pairs_override_builtin(self):
        cc = OpenCC()

        output = cc.detofu_with_custom_pairs(
            "𬴂",
            "all",
            [("𬴂", "马")],
        )

        assert output == "马"

    def test_cli_detofu_all(self):
        output = self._run_convert_cli("𠉂𪠟𫝈𫬐", detofu="all")

        assert output == "㒓㓄㑮㘔"

    def test_cli_detofu_ext_c(self):
        input_text = "𠉂𪠟𫝈𫬐"
        expected = OpenCC("s2t").detofu(OpenCC("s2t").convert(input_text), "ext-c")
        output = self._run_convert_cli(input_text, detofu="ext-c")

        assert output == expected

    def test_cli_detofu_all_with_custom_file(self):
        with TemporaryDirectory() as tmpdir:
            custom_path = Path(tmpdir) / "custom.txt"
            custom_path.write_text("𣭲\t氄\tB\n", encoding="utf-8")

            output = self._run_convert_cli(
                "𣭲毛",
                detofu="all",
                detofu_file=str(custom_path),
            )

        assert output == "氄毛"

    def test_cli_detofu_file_requires_detofu(self):
        with TemporaryDirectory() as tmpdir:
            input_path = Path(tmpdir) / "input.txt"
            output_path = Path(tmpdir) / "output.txt"
            custom_path = Path(tmpdir) / "custom.txt"
            input_path.write_text("𣭲毛", encoding="utf-8")
            custom_path.write_text("𣭲\t氄\tB\n", encoding="utf-8")

            rc = subcommand_convert(SimpleNamespace(
                config="s2t",
                input=str(input_path),
                output=str(output_path),
                punct=False,
                detofu=None,
                detofu_file=str(custom_path),
                in_enc="UTF-8",
                out_enc="UTF-8",
            ))

        assert rc == 1

    def _run_convert_cli(
            self,
            text: str,
            detofu: str,
            detofu_file: Optional[str] = None,
    ) -> str:
        with TemporaryDirectory() as tmpdir:
            input_path = Path(tmpdir) / "input.txt"
            output_path = Path(tmpdir) / "output.txt"
            input_path.write_text(text, encoding="utf-8")

            rc = subcommand_convert(SimpleNamespace(
                config="s2t",
                input=str(input_path),
                output=str(output_path),
                punct=False,
                detofu=detofu,
                detofu_file=detofu_file,
                in_enc="UTF-8",
                out_enc="UTF-8",
            ))

            assert rc == 0
            return output_path.read_text(encoding="utf-8")


if __name__ == '__main__':
    unittest.main()
