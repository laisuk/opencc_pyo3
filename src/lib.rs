// Import OpenCC crate and PyO3 for Python bindings
use once_cell::sync::Lazy;
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pyo3::exceptions;
use pyo3::prelude::*;
use std::collections::HashSet;

/// List of supported OpenCC conversion configurations.
/// These correspond to different Chinese conversion schemes, e.g.:
/// - "s2t": Simplified to Traditional
/// - "t2s": Traditional to Simplified
/// - "s2tw": Simplified to Traditional (Taiwan Standard)
/// - ...and others (see README for full list)
pub static CONFIG_SET: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "s2t", "t2s", "s2tw", "tw2s", "s2twp", "tw2sp", "s2hk", "hk2s", "t2tw", "tw2t", "t2twp",
        "tw2tp", "t2hk", "hk2t", "t2jp", "jp2t",
    ]
    .iter()
    .copied()
    .collect()
});

/// Python class wrapping the Rust OpenCC struct.
///
/// # Fields
/// - `opencc`: The internal Rust OpenCC instance.
/// - `config`: The conversion configuration string (e.g. "s2t").
///
/// # Python Attributes
/// - `config`: Get/set the current conversion configuration.
#[pyclass]
#[pyo3(subclass)]
struct OpenCC {
    opencc: _OpenCC,
    #[pyo3(get, set)]
    config: String,
    last_error: String,
}

/// Python methods for the OpenCC class.
#[pymethods]
impl OpenCC {
    /// Create a new OpenCC instance.
    ///
    /// # Arguments
    /// - `config` (optional): The conversion configuration string.
    ///   If not provided or invalid, defaults to "s2t".
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<&str>) -> Self {
        let opencc = _OpenCC::new();
        let (config_str, error_str) = match config {
            Some(c) if CONFIG_SET.contains(&c) => (c.to_string(), String::new()),
            Some(c) => (
                "s2t".to_string(),
                format!("Invalid config '{}', reverted to 's2t'", c),
            ),
            None => ("s2t".to_string(), String::new()),
        };
        OpenCC {
            opencc,
            config: config_str,
            last_error: error_str,
        }
    }

    /// Convert input text using the current configuration.
    ///
    /// # Arguments
    /// - `input_text`: The input string to convert.
    /// - `punctuation`: Whether to also convert punctuation.
    ///
    /// # Returns
    /// The converted string.
    fn convert(&self, input_text: &str, punctuation: bool) -> String {
        self.opencc.convert(input_text, &self.config, punctuation)
    }

    /// Detect the code of the input text.
    ///
    /// # Arguments
    /// - `input_text`: The input string to check.
    ///
    /// # Returns
    /// An integer code representing the detected text type.
    /// 1 - Traditional Chinese, 2 - Simplified Chinese, 0 - Others
    fn zho_check(&self, input_text: &str) -> i32 {
        self.opencc.zho_check(input_text)
    }

    /// Get the current configuration name.
    ///
    /// # Returns
    /// A string slice representing the currently active OpenCC config,
    /// such as `"s2t"`, `"t2s"`, etc.
    fn get_config(&self) -> &str {
        &self.config
    }

    /// Set the configuration name.
    ///
    /// # Arguments
    /// * `config` - A string slice representing the new configuration to set.
    ///              If the value is not in the supported list, it will fall back
    ///              to `"s2t"` and set an error message.
    ///
    /// # Behavior
    /// - If the config is valid, it will update the current config and clear any previous error.
    /// - If the config is invalid, it sets the config to `"s2t"` and stores an error in `last_error`.
    fn apply_config(&mut self, config: &str) {
        if CONFIG_SET.contains(config) {
            self.config = config.to_string();
            self.last_error.clear();
        } else {
            self.config = "s2t".to_string();
            self.last_error = format!("Invalid config '{}', reverted to 's2t'", config);
        }
    }

    /// Get the most recent error string (if any).
    ///
    /// # Returns
    /// A string slice containing the most recent error message.
    /// If no error occurred, returns an empty string.
    fn get_last_error(&self) -> &str {
        &self.last_error
    }

    /// Get a list of all supported OpenCC configuration codes.
    ///
    /// # Returns
    /// A vector of string slices representing valid configuration codes,
    /// such as `"s2t"`, `"t2s"`, `"s2tw"`, etc.
    #[staticmethod]
    fn supported_configs() -> Vec<&'static str> {
        CONFIG_SET.iter().copied().collect()
    }

    /// Check if a configuration name is valid.
    ///
    /// # Arguments
    /// * `config` - A string slice representing the config to check.
    ///
    /// # Returns
    /// `true` if the config is supported, otherwise `false`.
    #[staticmethod]
    fn is_valid_config(config: &str) -> bool {
        CONFIG_SET.contains(config)
    }
}

/// Extracts plain text from a PDF file.
///
/// This uses the pure-Rust `pdf-extract` crate. It works well for many PDFs,
/// but for tricky CJK encodings or missing ToUnicode maps you may want to
/// switch to a PDFium-based backend later.
///
/// Parameters
/// ----------
/// path : str
///     Path to the PDF file on disk.
///
/// Returns
/// -------
/// str
///     Concatenated text of all pages.
#[pyfunction]
fn extract_pdf_text(path: &str) -> PyResult<String> {
    let text = pdf_extract::extract_text(path).map_err(|e| {
        exceptions::PyRuntimeError::new_err(format!(
            "Failed to extract text from PDF '{}': {e}",
            path
        ))
    })?;
    Ok(text)
}

/// Reflow CJK paragraphs from PDF-extracted text.
///
/// This is a Rust/PyO3 port of `reflow_cjk_paragraphs_core()` from pdf_helper.py.
/// It merges artificial line breaks while保留段落、标题、章节行等结构。
///
/// Parameters
/// ----------
/// text : str
///     Raw text (usually from `extract_pdf_text()`).
/// add_pdf_page_header : bool
///     If `False`, try to skip page-break-like blank lines that are not
///     preceded by CJK punctuation. If `True`, keep those gaps.
/// compact : bool
///     If `True`, paragraphs are joined with a single newline ("p1\\np2").
///     If `False`, paragraphs are separated by a blank line ("p1\\n\\np2").
///
/// Returns
/// -------
/// str
///     Reflowed text.
#[pyfunction]
fn reflow_cjk_paragraphs(text: &str, add_pdf_page_header: bool, compact: bool) -> PyResult<String> {
    // If the whole text is whitespace, return as-is.
    if text.chars().all(|c| c.is_whitespace()) {
        return Ok(text.to_owned());
    }

    // Normalize line endings
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines = normalized.split('\n');

    let mut segments: Vec<String> = Vec::new();
    let mut buffer = String::new();
    let mut dialog_state = DialogState::new();

    for raw_line in lines {
        // 1) normalize trailing whitespace, then strip only HALF-width indent;
        //    keep full-width indent (U+3000) for CJK paragraph styling.
        let trimmed_end = raw_line.trim_end();
        let stripped = strip_halfwidth_indent_keep_fullwidth(trimmed_end);

        // For heading detection (前言 / 第X章...) we want a fully left-trimmed probe.
        let heading_probe = stripped.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');

        // Treat lines that are effectively blank as paragraph separators
        if heading_probe.trim().is_empty() {
            if !add_pdf_page_header && !buffer.is_empty() {
                if let Some(last_char) = buffer.chars().rev().find(|c| !c.is_whitespace()) {
                    // Page-break-like blank line without ending punctuation → skip
                    if !CJK_PUNCT_END.contains(&last_char) {
                        continue;
                    }
                }
            }

            // End of a paragraph → flush buffer, don't emit an empty segment
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            continue;
        }

        // 2) Page markers like "=== [Page 1/20] ==="
        if is_page_marker(heading_probe) {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(stripped.to_owned());
            continue;
        }

        // 3) Title heading (前言, 序章, 第xxx章, etc.)
        let is_title_heading = is_title_heading_line(heading_probe);
        let line_text = if is_title_heading {
            collapse_repeated_segments(stripped)
        } else {
            stripped.to_owned()
        };

        if is_title_heading {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(line_text.clone());
            continue;
        }

        // --- NEW: dialog-start detection (like C#/Python) ---
        let current_is_dialog_start = is_dialog_start(&line_text);

        // 4) First line of a new paragraph
        if buffer.is_empty() {
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        let buffer_text = &buffer;

        // If this line *starts* a dialog, always flush previous paragraph.
        if current_is_dialog_start {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // --- colon + dialog continuation ---
        // e.g. "她寫了一行字：" + "「如果連自己都不相信，那就沒救了。」"
        if let Some(last_char) = buffer_text.chars().rev().find(|c| !c.is_whitespace()) {
            if last_char == '：' || last_char == ':' {
                if let Some(first_ch) = line_text.chars().next() {
                    if DIALOG_OPENERS.contains(&first_ch) {
                        buffer.push_str(&line_text);
                        dialog_state.update(&line_text);
                        continue;
                    }
                }
            }
        }

        // NOTE: we no longer block splits just because dialog is "unclosed".
        // Dialog paragraphs can still end normally on CJK punctuation.

        // 5) Buffer ends with CJK punctuation → finalize paragraph, start new one
        if buffer_ends_with_cjk_punct(buffer_text) {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 6) Previous buffer looks like a heading-like short title
        if is_heading_like(buffer_text) {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 7) Chapter-like endings: 章 / 节 / 部 / 卷 (with trailing brackets)
        if is_chapter_ending_line(buffer_text) {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 8) Default: merge as soft line break
        buffer.push_str(&line_text);
        dialog_state.update(&line_text);
    }

    // Flush last buffer
    if !buffer.is_empty() {
        segments.push(buffer);
    }

    let result = if compact {
        segments.join("\n")
    } else {
        segments.join("\n\n")
    };

    Ok(result)
}
// ---------------------------------------------------------------------------
// Pure-Rust helpers for ultra-fast CJK reflow (no regex).
// ---------------------------------------------------------------------------

const CJK_PUNCT_END: &[char] = &[
    '。', '！', '？', '；', '：', '…', '—', '”', '」', '’', '』', '）', '】', '》', '〗', '〔',
    '〕', '〉', '］', '｝', '》', '.', '?', '!',
];

// Closing brackets that can trail after a chapter marker (章/节/部/卷/節)
const CHAPTER_TRAIL_BRACKETS: &[char] = &['】', '》', '〗', '〕', '〉', '」', '』', '）'];

// Keywords treated as headings even without "第…章"
const HEADING_KEYWORDS: &[&str] = &[
    "前言", "序章", "终章", "尾声", "后记", "番外", "尾聲", "後記",
];

const CHAPTER_MARKERS: &[char] = &['章', '节', '部', '卷', '節', '回'];

#[allow(dead_code)]
fn is_cjk_char(ch: char) -> bool {
    let u = ch as u32;
    // Basic CJK + extensions + compatibility (rough but cheap)
    (0x3400..=0x9FFF).contains(&u)    // CJK Unified Ideographs
        || (0xF900..=0xFAFF).contains(&u) // CJK Compatibility Ideographs
        || (0x20000..=0x2EBEF).contains(&u) // Ext-B..Ext-F-ish
}

#[allow(dead_code)]
fn has_any_cjk(s: &str) -> bool {
    s.chars().any(is_cjk_char)
}

fn buffer_ends_with_cjk_punct(s: &str) -> bool {
    if let Some(ch) = s.chars().rev().find(|c| !c.is_whitespace()) {
        CJK_PUNCT_END.contains(&ch)
    } else {
        false
    }
}

fn is_page_marker(s: &str) -> bool {
    s.starts_with("=== ") && s.ends_with("===")
}

/// Heading detection (前言/序章/终章/尾声/后记/番外, or 第…章/节/部/卷).
/// Length constraint (<= 60 chars) enforced here instead of via regex.
fn is_title_heading_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Length ≤ 60 chars
    if s.chars().count() > 60 {
        return false;
    }

    // Direct keyword match at start: 前言, 序章, 终章, 尾声, 后记, 番外, 尾聲, 後記
    for &kw in HEADING_KEYWORDS {
        if s.starts_with(kw) {
            return true;
        }
    }

    // Pattern: 第 ... 章/节/部/卷/節/回 (within first ~12 chars)
    if s.starts_with('第') {
        for (i, ch) in s.chars().enumerate() {
            if CHAPTER_MARKERS.contains(&ch) {
                return i <= 12; // chapter marker must be early
            }
            if i > 12 {
                return false; // too far, bail early
            }
        }
        return false;
    }

    false
}

/// Chapter-like ending: line ≤ 15 chars and last non-bracket char is 章/节/部/卷/節.
fn is_chapter_ending_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    if s.chars().count() > 15 {
        return false;
    }

    // Strip trailing brackets like 】》〗〕〉」』）
    let mut trimmed = s;
    loop {
        if let Some(last) = trimmed.chars().last() {
            if CHAPTER_TRAIL_BRACKETS.contains(&last) {
                // Drop last char
                trimmed = &trimmed[..trimmed.len() - last.len_utf8()];
                continue;
            }
        }
        break;
    }

    // Now check final character using unified chapter marker list
    if let Some(last) = trimmed.chars().last() {
        CHAPTER_MARKERS.contains(&last)
    } else {
        false
    }
}

fn is_dialog_start(s: &str) -> bool {
    // ignore leading half/full-width spaces
    let trimmed = s.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
    if let Some(ch) = trimmed.chars().next() {
        DIALOG_OPENERS.contains(&ch)
    } else {
        false
    }
}

/// Heading-like: short, mostly CJK, no CJK end punctuation, not page marker.
fn is_heading_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // page markers like "=== [Page 1/20] ===" are NOT headings
    if is_page_marker(s) {
        return false;
    }

    // If contains CJK end punctuation anywhere, not heading/emphasis
    if s.chars().any(|ch| CJK_PUNCT_END.contains(&ch)) {
        return false;
    }

    // If line has an opening bracket but no closing bracket,
    // it's most likely a broken parenthetical, NOT a standalone heading.
    let has_open = s.chars().any(|ch| OPEN_BRACKETS.contains(&ch));
    let has_close = s.chars().any(|ch| CLOSE_BRACKETS.contains(&ch));
    if has_open && !has_close {
        return false;
    }

    // Count logical characters (not bytes)
    let len = s.chars().count();

    // Rule A: short CJK or mixed lines (≤15)
    if len <= 15
        && s.chars().any(|ch| (ch as u32) > 0x7F)  // contains at least one CJK
        && !matches!(s.chars().last(), Some('，' | ','))
    {
        return true;
    }

    // Rule B: short pure-Latin emphasis (≤15), must contain at least one letter
    if len <= 15
        && s.chars().all(|ch| (ch as u32) <= 0x7F) // pure ASCII
        && s.chars().any(|ch| ch.is_ascii_alphabetic())
    {
        return true;
    }

    false
}

/// Collapse repeated tokens like "第一章第一章第一章" → "第一章".
/// Port of collapse_repeated_token() idea, but without regex.
fn collapse_repeated_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    let length = chars.len();

    if length < 4 || length > 200 {
        return token.to_owned();
    }

    for unit_len in 2..=20 {
        if unit_len > length / 2 {
            break;
        }
        if length % unit_len != 0 {
            continue;
        }

        let unit = &chars[0..unit_len];
        let repeat_count = length / unit_len;
        let mut all_match = true;

        for i in 1..repeat_count {
            let start = i * unit_len;
            let end = start + unit_len;
            if &chars[start..end] != unit {
                all_match = false;
                break;
            }
        }

        if all_match {
            return unit.iter().collect();
        }
    }

    token.to_owned()
}

/// Collapse repeated segments in a line (split+collapse tokens, join with single spaces).
fn collapse_repeated_segments(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_owned();
    }

    // split_whitespace() already collapses multiple spaces/tabs
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return line.to_owned();
    }

    let collapsed_parts: Vec<String> = parts.into_iter().map(collapse_repeated_token).collect();

    collapsed_parts.join(" ")
}

// Dialog openers (Simplified / Traditional / JP-style)
const DIALOG_OPENERS: &[char] = &['“', '‘', '「', '『'];

// Bracket sets for unmatched-bracket suppression in headings
const OPEN_BRACKETS: &[char] = &['（', '(', '[', '【', '《'];
const CLOSE_BRACKETS: &[char] = &['）', ')', ']', '】', '》'];

/// Track unmatched dialog brackets for the current paragraph buffer.
/// Incremental update → no need to rescan the whole buffer each time.
struct DialogState {
    double_quote: i32, // “ ”
    single_quote: i32, // ‘ ’
    corner: i32,       // 「 」
    corner_bold: i32,  // 『 』
}

impl DialogState {
    fn new() -> Self {
        Self {
            double_quote: 0,
            single_quote: 0,
            corner: 0,
            corner_bold: 0,
        }
    }

    fn reset(&mut self) {
        self.double_quote = 0;
        self.single_quote = 0;
        self.corner = 0;
        self.corner_bold = 0;
    }

    fn update(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '“' => self.double_quote += 1,
                '”' => {
                    if self.double_quote > 0 {
                        self.double_quote -= 1;
                    }
                }
                '‘' => self.single_quote += 1,
                '’' => {
                    if self.single_quote > 0 {
                        self.single_quote -= 1;
                    }
                }
                '「' => self.corner += 1,
                '」' => {
                    if self.corner > 0 {
                        self.corner -= 1;
                    }
                }
                '『' => self.corner_bold += 1,
                '』' => {
                    if self.corner_bold > 0 {
                        self.corner_bold -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    #[allow(dead_code)]
    fn is_unclosed(&self) -> bool {
        self.double_quote > 0 || self.single_quote > 0 || self.corner > 0 || self.corner_bold > 0
    }
}

/// Strip only *halfwidth* leading spaces, keep fullwidth `\u3000`
/// (so CJK paragraph indentation survives, markdown stays clean).
fn strip_halfwidth_indent_keep_fullwidth(s: &str) -> &str {
    let mut start_byte = 0;
    for (idx, ch) in s.char_indices() {
        if ch == ' ' {
            // skip halfwidth spaces
            start_byte = idx + ch.len_utf8();
            continue;
        }
        // stop on first non-halfwidth-space (including fullwidth indent)
        break;
    }
    &s[start_byte..]
}

/// Python module definition for opencc_pyo3.
/// Exposes the OpenCC class to Python.
#[pymodule]
fn opencc_pyo3(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<OpenCC>()?;
    m.add_function(wrap_pyfunction!(extract_pdf_text, m)?)?;
    m.add_function(wrap_pyfunction!(reflow_cjk_paragraphs, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test for the zho_check method.
    #[test]
    fn test_zho_check() {
        let opencc = OpenCC::new(Option::from(""));
        let text = "春眠不觉晓，处处闻啼鸟";
        let text_code = opencc.zho_check(text);
        let expected = 2;
        assert_eq!(text_code, expected);
    }

    // Test for supported config list
    #[test]
    fn test_get_supported_list() {
        let configs = OpenCC::supported_configs();
        let expected: HashSet<&str> = CONFIG_SET.iter().copied().collect();
        let actual: HashSet<&str> = configs.into_iter().collect();
        assert_eq!(actual, expected);
    }

    /// Test PDF text extraction using a known CJK PDF.
    /// Saves *reflowed* text to `tests/简体字_output.txt` for manual inspection.
    #[test]
    fn test_extract_pdf_text() {
        use std::fs;
        use std::io::Write;
        use std::path::Path;

        // PDF input (relative to crate root)
        let input_path = "tests/简体字.pdf";

        assert!(
            Path::new(input_path).exists(),
            "Test PDF not found at: {}. Make sure the file exists.",
            input_path
        );

        // Extract text
        let text = extract_pdf_text(input_path).expect("Failed to extract text from test PDF");

        // Sanity check: extracted text should not be empty
        assert!(
            !text.trim().is_empty(),
            "PDF extraction returned empty text"
        );

        // Ensure some CJK characters appear (adjust if your sample PDF differs)
        assert!(
            text.contains("字") || text.contains("简") || text.contains("体"),
            "Extracted text does not contain expected CJK characters.\nGot: {}",
            text
        );

        // 🔹 Reflow CJK paragraphs before saving
        // add_pdf_page_header = false, compact = false (blank line between paragraphs)
        let reflowed =
            reflow_cjk_paragraphs(&text, false, false).expect("Failed to reflow CJK paragraphs");

        // Save output to file for manual review
        let output_path = "tests/简体字_output.txt";
        let mut file = fs::File::create(output_path).expect("Failed to create output .txt file");

        file.write_all(reflowed.as_bytes())
            .expect("Failed to write extracted text to output file");

        // Optional: check output file exists and is non-empty
        let out_meta = fs::metadata(output_path).expect("Failed to stat output file");
        assert!(out_meta.len() > 0, "Output text file is empty");
    }
}
