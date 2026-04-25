import sys
import unittest
from unittest import mock
import zipfile
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT_DIR = Path(__file__).resolve().parents[1]
PYTHON_SRC_DIR = ROOT_DIR / "python"
if str(PYTHON_SRC_DIR) not in sys.path:
    sys.path.insert(0, str(PYTHON_SRC_DIR))

from opencc_pyo3 import OpenCC
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
        # v0.9.2 C API reports invalid-config origin during construction,
        # while apply_config fallback no longer populates last_error.
        self.assertEqual(cc.get_last_error(), "")

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


if __name__ == '__main__':
    unittest.main()
