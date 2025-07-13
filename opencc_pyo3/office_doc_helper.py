"""
OpenCC-based Office and EPUB document converter.

This module provides helper functions to convert and repackage Office documents and EPUBs,
supporting optional font preservation.

Supported formats: docx, xlsx, pptx, odt, ods, odp, epub.

Author
------
Laisuk Lai (https://github.com/laisuk)
"""

import os
import re
import shutil
import tempfile
import zipfile
from typing import Tuple, List, Optional

# Global list of supported Office document formats
OFFICE_FORMATS = [
    "docx",  # Word
    "xlsx",  # Excel
    "pptx",  # PowerPoint
    "odt",  # OpenDocument Text
    "ods",  # OpenDocument Spreadsheet
    "odp",  # OpenDocument Presentation
    "epub",  # eBook (XHTML-based)
]


def convert_office_doc(
        input_path: str,
        output_path: str,
        office_format: str,
        converter,
        punctuation: bool = False,
        keep_font: bool = False,
) -> Tuple[bool, str]:
    """
    Converts an Office document by applying OpenCC conversion on specific XML parts.
    Optionally preserves original_value font names to prevent them from being altered.

    Args:
        input_path: Path to input .docx, .xlsx, .pptx, .odt, .epub, etc.
        output_path: Path for the output converted document.
        office_format: One of 'docx', 'xlsx', 'pptx', 'odt', 'ods', 'odp', 'epub'.
        converter: An object with a method `convert(text, punctuation=True|False)`.
        punctuation: Whether to convert punctuation.
        keep_font: If True, font names are preserved during conversion.

    Returns:
        (success: bool, message: str)
    """
    temp_dir = os.path.join(tempfile.gettempdir(), f"{office_format}_temp_{os.urandom(6).hex()}")

    try:
        with zipfile.ZipFile(input_path, 'r') as archive:
            archive.extractall(temp_dir)

        target_paths = _get_target_xml_paths(office_format, temp_dir)

        if not target_paths:
            return False, f"❌ Unsupported or invalid format: {office_format}"

        converted_count = 0

        for relative_path in target_paths:
            full_path = os.path.join(temp_dir, relative_path)
            if not os.path.isfile(full_path):
                continue

            with open(full_path, "r", encoding="utf-8") as f:
                xml_content = f.read()

            font_map = {}
            if keep_font:
                pattern = _get_font_regex_pattern(office_format)
                font_map = {}
                font_counter = 0

                if pattern:
                    # Replace font-family values with unique markers to preserve them during conversion
                    def replace_font(match):
                        nonlocal font_counter
                        font_key = f"__F_O_N_T_{font_counter}__"
                        original_value = match.group(2)
                        font_map[font_key] = original_value
                        font_counter += 1
                        return f"{match.group(1)}{font_key}{match.group(3)}"

                    xml_content = re.sub(pattern, replace_font, xml_content)

            converted = converter.convert(xml_content, punctuation=punctuation)

            if keep_font:
                for marker, original in font_map.items():
                    converted = converted.replace(marker, original)

            with open(full_path, "w", encoding="utf-8") as f:
                f.write(converted)

            converted_count += 1

        if converted_count == 0:
            return False, f"⚠️ No valid XML fragments were found. Is the format '{office_format}' correct?"

        if os.path.exists(output_path):
            os.remove(output_path)

        if office_format == "epub":
            return create_epub_zip_with_spec(temp_dir, output_path)
        else:
            with zipfile.ZipFile(output_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                for root, _, files in os.walk(temp_dir):
                    for file in files:
                        full_path = os.path.join(root, file)
                        arc_name = os.path.relpath(full_path, temp_dir).replace("\\", "/")
                        archive.write(full_path, arc_name)

        return True, f"✅ Successfully converted {converted_count} fragment(s) in {office_format} document."

    except Exception as ex:
        return False, f"❌ Conversion failed: {ex}"
    finally:
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)


def _get_target_xml_paths(office_format: str, base_dir: str) -> Optional[List[str]]:
    """
    Returns a list of XML file paths within the extracted Office/EPUB directory
    that should be converted for the given format.

    Args:
        office_format: The document format (e.g., 'docx', 'xlsx', 'epub').
        base_dir: The root directory of the extracted archive.

    Returns:
        List of relative XML file paths to process, or None if unsupported.
    """
    if office_format == "docx":
        return [os.path.join("word", "document.xml")]
    elif office_format == "xlsx":
        return [os.path.join("xl", "sharedStrings.xml")]
    elif office_format == "pptx":
        ppt_dir = os.path.join(base_dir, "ppt")
        if os.path.isdir(ppt_dir):
            return [
                os.path.relpath(os.path.join(root, file), base_dir)
                for root, _, files in os.walk(ppt_dir)
                for file in files
                if file.endswith(".xml") and (
                        file.startswith("slide")
                        or "notesSlide" in file
                        or "slideMaster" in file
                        or "slideLayout" in file
                        or "comment" in file
                )
            ]
    elif office_format in ("odt", "ods", "odp"):
        return ["content.xml"]
    elif office_format == "epub":
        return [
            os.path.relpath(os.path.join(root, file), base_dir)
            for root, _, files in os.walk(base_dir)
            for file in files
            if file.lower().endswith((".xhtml", ".opf", ".ncx"))
        ]
    return None


def _get_font_regex_pattern(office_format: str) -> Optional[str]:
    """
    Returns a regex pattern to match font-family attributes for the given format.

    Args:
        office_format: The document format.

    Returns:
        Regex string or None if not applicable.
    """
    return {
        "docx": r'(w:(?:eastAsia|ascii|hAnsi|cs)=")([^"]+)(")',
        "xlsx": r'(val=")(.*?)(")',
        "pptx": r'(typeface=")(.*?)(")',
        "odt": r'((?:style:font-name(?:-asian|-complex)?|svg:font-family|style:name)=["\'])([^"\']+)(["\'])',
        "ods": r'((?:style:font-name(?:-asian|-complex)?|svg:font-family|style:name)=["\'])([^"\']+)(["\'])',
        "odp": r'((?:style:font-name(?:-asian|-complex)?|svg:font-family|style:name)=["\'])([^"\']+)(["\'])',
        "epub": r'(font-family\s*:\s*)([^;]+)([;])?',
    }.get(office_format)


def create_epub_zip_with_spec(source_dir: str, output_path: str) -> Tuple[bool, str]:
    """
    Creates a valid EPUB-compliant ZIP archive.
    Ensures `mimetype` is the first file and uncompressed.

    Args:
        source_dir: The unpacked EPUB directory.
        output_path: Final path to .epub file.

    Returns:
        Tuple of (success, message)
    """
    mime_path = os.path.join(source_dir, "mimetype")

    try:
        with zipfile.ZipFile(output_path, "w") as epub:
            if os.path.isfile(mime_path):
                epub.write(mime_path, "mimetype", compress_type=zipfile.ZIP_STORED)
            else:
                return False, "❌ 'mimetype' file is missing. EPUB requires it as the first entry."

            for root, _, files in os.walk(source_dir):
                for file in files:
                    full_path = os.path.join(root, file)
                    if os.path.samefile(full_path, mime_path):
                        continue
                    arc_name = os.path.relpath(full_path, source_dir).replace("\\", "/")
                    epub.write(full_path, arc_name, compress_type=zipfile.ZIP_DEFLATED)

        return True, "✅ EPUB archive created successfully."
    except Exception as ex:
        return False, f"❌ Failed to create EPUB: {ex}"
