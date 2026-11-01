import ctypes
import importlib
import io
import sys
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from types import SimpleNamespace
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from opencc_pyo3 import pdfium_loader
from opencc_pyo3.__main__ import subcommand_pdf


class TestPdfRegressions(unittest.TestCase):
    def helper(self):
        with mock.patch.object(pdfium_loader, "load_pdfium", return_value=mock.MagicMock()):
            return importlib.import_module("opencc_pyo3.pdfium_helper")

    def test_windows_architectures(self):
        for machine, bits, expected in (
                ("AMD64", 64, "win-x64"), ("x86", 32, "win-x86"),
                ("ARM64", 64, "win-arm64"), ("aarch64", 64, "win-arm64"),
                ("AMD64", 32, "win-x86"),
        ):
            with self.subTest(machine=machine, bits=bits), \
                    mock.patch.object(pdfium_loader.sys, "platform", "win32"), \
                    mock.patch.object(pdfium_loader.sys, "maxsize", 2 ** (bits - 1) - 1), \
                    mock.patch.object(pdfium_loader.platform, "machine", return_value=machine):
                self.assertEqual(pdfium_loader._detect_platform_folder(), expected)

    def test_cleanup_failures(self):
        helper = self.helper()
        for failure in ("document", "load", "textpage", "count", "extract", "callback"):
            with self.subTest(failure=failure):
                api = mock.MagicMock()
                api.FPDF_LoadDocument.return_value = 1
                api.FPDF_GetPageCount.return_value = 1
                api.FPDF_LoadPage.return_value = 2
                api.FPDFText_LoadPage.return_value = 3
                api.FPDFText_CountChars.return_value = 1
                api.FPDFText_GetText.return_value = 0
                callback = mock.Mock()
                if failure == "document":
                    api.FPDF_LoadDocument.return_value = None
                elif failure == "callback":
                    callback.side_effect = RuntimeError("callback")
                else:
                    name = {"load": "FPDF_LoadDocument", "textpage": "FPDFText_LoadPage",
                            "count": "FPDFText_CountChars", "extract": "FPDFText_GetText"}[failure]
                    getattr(api, name).side_effect = RuntimeError(failure)
                with mock.patch.object(helper, "_pdfium", api), self.assertRaises(RuntimeError):
                    helper.extract_pdf_pages_with_callback_pdfium("test.pdf", callback)
                api.FPDF_DestroyLibrary.assert_called_once_with()
                if failure in ("document", "load"):
                    api.FPDF_CloseDocument.assert_not_called()
                    api.FPDF_ClosePage.assert_not_called()
                else:
                    api.FPDF_CloseDocument.assert_called_once_with(1)
                    api.FPDF_ClosePage.assert_called_once_with(2)
                if failure in ("count", "extract", "callback"):
                    api.FPDFText_ClosePage.assert_called_once_with(3)
                else:
                    api.FPDFText_ClosePage.assert_not_called()

    def test_decoder_contract(self):
        decode = self.helper()._decode_pdfium_buffer
        text = "甲\0乙\r\n\n\n丙"
        units = text.encode("utf-16le") + b"\0\0"
        buf = (ctypes.c_uint16 * (len(units) // 2)).from_buffer_copy(units)
        self.assertEqual(decode(buf, len(buf)), "甲\0乙\n\n丙")
        self.assertEqual(decode((ctypes.c_uint16 * 1)(0), 1), "\n")
        self.assertEqual(decode(buf, 0), "")

    def test_null_page_handles(self):
        helper = self.helper()
        for missing in ("page", "textpage"):
            with self.subTest(missing=missing):
                api = mock.MagicMock()
                api.FPDF_LoadDocument.return_value = 1
                api.FPDF_GetPageCount.return_value = 1
                api.FPDF_LoadPage.return_value = None if missing == "page" else 2
                api.FPDFText_LoadPage.return_value = None
                callback = mock.Mock(side_effect=RuntimeError("callback"))
                with mock.patch.object(helper, "_pdfium", api), self.assertRaises(RuntimeError):
                    helper.extract_pdf_pages_with_callback_pdfium("test.pdf", callback, True)
                callback.assert_called_once_with(1, 1, "=== [Page 1/1] ===\n\n")
                api.FPDFText_ClosePage.assert_not_called()
                if missing == "page":
                    api.FPDF_ClosePage.assert_not_called()
                else:
                    api.FPDF_ClosePage.assert_called_once_with(2)
                api.FPDF_CloseDocument.assert_called_once_with(1)
                api.FPDF_DestroyLibrary.assert_called_once_with()

    def test_cli_transactional_output(self):
        helper = self.helper()
        for failure in (None, "write", "replace", "create"):
            with self.subTest(failure=failure), TemporaryDirectory() as directory:
                root = Path(directory)
                source, output = root / "input.pdf", root / "out.txt"
                source.touch()
                output.write_bytes(b"original")
                args = SimpleNamespace(input=str(source), output=str(output), extract=True,
                                       timing=False, header=False, reflow=False, config=None,
                                       punct=False, norm_compat=False, norm_compat_extended=False,
                                       detofu=None, detofu_file=None, custom_dict=None)
                def extract(path, callback, header):
                    callback(1, 1, "甲\n\n")
                with mock.patch.object(helper, "extract_pdf_pages_with_callback_pdfium", side_effect=extract), \
                        redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
                    if failure == "write":
                        # Closing flushes the real sibling file; fail after a partial write.
                        import tempfile
                        original = tempfile.NamedTemporaryFile
                        def broken_writer(*a, **kw):
                            stream = original(*a, **kw)
                            stream.write("partial")
                            stream.write = mock.Mock(side_effect=OSError("disk full"))
                            return stream
                        with mock.patch("opencc_pyo3.__main__.tempfile.NamedTemporaryFile", side_effect=broken_writer):
                            rc = subcommand_pdf(args)
                    elif failure == "replace":
                        with mock.patch("opencc_pyo3.__main__.os.replace", side_effect=OSError("denied")):
                            rc = subcommand_pdf(args)
                    elif failure == "create":
                        with mock.patch("opencc_pyo3.__main__.tempfile.NamedTemporaryFile", side_effect=OSError("denied")):
                            rc = subcommand_pdf(args)
                    else:
                        rc = subcommand_pdf(args)
                self.assertEqual(rc, 1 if failure else 0)
                self.assertEqual(output.read_bytes(), b"original" if failure else "甲\n\n".encode("utf-8"))
                self.assertEqual(sorted(p.name for p in root.iterdir()), ["input.pdf", "out.txt"])
