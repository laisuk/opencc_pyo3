// Import OpenCC crate and PyO3 for Python bindings
use once_cell::sync::Lazy;
use opencc_fmmseg;
use opencc_fmmseg::OpenCC as _OpenCC;
use pdf_extract::Document;
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

/// Extracts plain text from a PDF file, split by pages.
///
/// This uses the pure-Rust `pdf-extract` crate. It returns one string per page,
/// in reading order. This is useful if you want to show a progress bar while
/// processing each page sequentially in Python.
///
/// Parameters
/// ----------
/// path : str
///     Path to the PDF file on disk.
///
/// Returns
/// -------
/// List[str]
///     A list of page texts. `result[i]` is the text of page `i + 1`.
#[pyfunction]
fn extract_pdf_text_pages(path: &str) -> PyResult<Vec<String>> {
    let pages = pdf_extract::extract_text_by_pages(path).map_err(|e| {
        exceptions::PyRuntimeError::new_err(format!(
            "Failed to extract text by pages from PDF '{}': {e}",
            path
        ))
    })?;
    Ok(pages)
}

/// Extracts PDF text page-by-page and reports progress to a Python callback.
///
/// For PDFs where `pdf-extract` can see the page tree:
///   - iterates real pages, including blank ones (blank → "").
/// For PDFs where `get_pages()` returns empty:
///   - falls back to `extract_text(path)` and calls the callback once as 1/1.
///
/// callback signature: callback(page_number, total_pages, text)
#[pyfunction]
fn extract_pdf_pages_with_callback(path: &str, callback: Py<PyAny>) -> PyResult<()> {
    let doc = Document::load(path).map_err(|e| {
        exceptions::PyRuntimeError::new_err(format!("Failed to open PDF '{}': {e}", path))
    })?;

    let pages = doc.get_pages();
    let total_pages = pages.len();

    fn normalize_page_text(mut s: String) -> String {
        if s.contains('\r') {
            s = s.replace("\r\n", "\n").replace('\r', "\n");
        }
        if s.trim().is_empty() {
            return "\n".to_string();
        }
        let t = s.trim().to_string();
        format!("{t}\n\n")
    }

    // Fallback: 0-page tree => single chunk
    if total_pages == 0 {
        eprintln!(
            "Warning: pdf-extract reports 0 pages for '{}'; falling back to single-chunk extract_text().",
            path
        );

        let text = pdf_extract::extract_text(path).map_err(|e| {
            exceptions::PyRuntimeError::new_err(format!(
                "Failed to extract text from PDF '{}': {e}",
                path
            ))
        })?;

        if text.trim().is_empty() {
            return Err(exceptions::PyRuntimeError::new_err(format!(
                "Pure-Rust pdf-extract could not extract any text from '{}'. This PDF likely requires a PDFium-based engine.",
                path
            )));
        }

        let text = normalize_page_text(text);

        return Python::attach(|py| {
            callback.call1(py, (1usize, 1usize, text))?;
            Ok(())
        });
    }

    // Normal path
    let page_numbers: Vec<u32> = pages.keys().copied().collect();

    Python::attach(move |py| -> PyResult<()> {
        for (idx, page_number) in page_numbers.iter().copied().enumerate() {
            let raw = match doc.extract_text(&[page_number]) {
                Ok(t) => t,
                Err(e) => {
                    eprintln!(
                        "Warning: failed to extract text from page {} of '{}': {} — treating as blank page.",
                        page_number, path, e
                    );
                    String::new()
                }
            };

            let text = normalize_page_text(raw);

            // 1-based page index for callback, consistent with your PDFium ctypes
            let page_1_based = idx + 1;
            callback.call1(py, (page_1_based, total_pages, text))?;
        }
        Ok(())
    })
}

// ---------------------------------------------------------------------------
// CJK PDF Reflow Engine (Rust implementation for opencc_pyo3)
// ---------------------------------------------------------------------------

/// Reflow CJK paragraphs from PDF-extracted text.
///
/// This merges artificial line breaks while preserving paragraphs,
/// headings, chapter lines, and dialog structure.
///
/// Parameters
/// ----------
/// text : &str
///     Raw text (usually from `extract_pdf_text()`).
/// add_pdf_page_header : bool
///     If `false`, try to skip page-break-like blank lines that are not
///     preceded by CJK punctuation. If `true`, keep those gaps.
/// compact : bool
///     If `true`, paragraphs are joined with a single newline ("p1\\np2").
///     If `false`, paragraphs are separated by a blank line ("p1\\n\\np2").
///
/// Returns
/// -------
/// String
///     Reflowed text.
///
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
        // 1) Visual form: trim right-side whitespace, then remove halfwidth indent
        let trimmed_end = raw_line.trim_end();
        let stripped_visual = strip_halfwidth_indent_keep_fullwidth(trimmed_end);

        // 1.1) Logical probe for heading detection (no left indent)
        let probe = stripped_visual.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');

        // 1.2 Visual divider line (box drawing / ---- / === / *** / ★★★ etc.)
        // Always force paragraph breaks.
        if is_box_drawing_line(probe) {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(stripped_visual.to_string());
            continue;
        }

        // 2) Collapse style-layer repeated segments *per line*:
        //    e.g. "麒麟三, 麒麟三, 麟麟三, 麒麟三" or
        //         "（第一季大结局） （第一季大结局） ..."
        //    This must run before heading / metadata detection.
        let line_text = collapse_repeated_segments(stripped_visual);

        // 3) Logical probe for heading detection (no left indent)
        let heading_probe = line_text.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');

        // 4) Empty line handling ---------------------------------------------
        if heading_probe.trim().is_empty() {
            if !add_pdf_page_header && !buffer.is_empty() {
                // Skip blank lines that look like soft page breaks if the
                // previous paragraph does not end with CJK punctuation.
                if let Some(last_char) = buffer.chars().rev().find(|c| !c.is_whitespace()) {
                    if !CJK_PUNCT_END.contains(&last_char) {
                        continue;
                    }
                }
            }

            // End of a paragraph → flush buffer, but do not add an empty segment.
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            continue;
        }

        // 5) Page marker lines ("=== [X/Y] ===") -----------------------------
        if is_page_marker(heading_probe) {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(line_text.clone());
            continue;
        }

        // 6) Heading / metadata detection ------------------------------------
        let is_title_heading = is_title_heading_line(heading_probe);
        let is_short_heading = is_heading_like(&line_text);
        let is_metadata = is_metadata_line(&line_text);

        let flush_buffer_and_push_line =
            |buffer: &mut String,
             segments: &mut Vec<String>,
             dialog_state: &mut DialogState,
             line_text: &str| {
                if !buffer.is_empty() {
                    segments.push(std::mem::take(buffer));
                    dialog_state.reset();
                }
                segments.push(line_text.to_owned());
            };

        // 6a) Metadata lines (key: value, e.g. "書名：xxx", "作者：yyy")
        if is_metadata {
            flush_buffer_and_push_line(&mut buffer, &mut segments, &mut dialog_state, &line_text);
            continue;
        }

        // 6b) Strong title headings (e.g. "第X章", "前言", "終章", etc.)
        if is_title_heading {
            flush_buffer_and_push_line(&mut buffer, &mut segments, &mut dialog_state, &line_text);
            continue;
        }

        if is_short_heading {
            let stripped = heading_probe; // or `stripped_visual` depending on your pipeline

            if !buffer.is_empty() {
                let buf_text = buffer.as_str();

                // 1) Unclosed bracket => continuation
                if has_unclosed_bracket(buf_text) {
                    // fall through (merge logic)
                } else {
                    let bt = buf_text.trim_end();
                    if let Some(last) = bt.chars().last() {
                        // 2) Comma-like => continuation
                        if last == '，' || last == ',' || last == '、' {
                            // fall through
                        } else {
                            // 3) all-CJK short heading requires prev line ended with sentence terminator
                            let is_all_cjk = is_all_cjk_ignoring_ws(stripped);
                            if is_all_cjk && !CJK_PUNCT_END.contains(&last) {
                                // fall through
                            } else {
                                segments.push(std::mem::take(&mut buffer));
                                dialog_state.reset();
                                segments.push(line_text.clone()); // or stripped.to_string()
                                continue;
                            }
                        }
                    } else {
                        // buffer is whitespace-only => treat as heading
                        segments.push(line_text.clone());
                        continue;
                    }
                }
            } else {
                segments.push(line_text.clone());
                continue;
            }
        }

        // 7) Dialog detection -------------------------------------------------
        let current_is_dialog_start = is_dialog_start(&line_text);

        // First line of a new paragraph
        if buffer.is_empty() {
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        let buffer_text = &buffer;

        // NEW RULE: if previous line ends with a comma, do NOT flush even if
        // this line starts with a dialog opener. Comma-ending usually means
        // the sentence is not finished.
        if current_is_dialog_start {
            let trimmed_buffer = buffer_text.trim_end();
            let last = trimmed_buffer.chars().rev().next();
            if let Some(ch) = last {
                if ch != '，' && ch != ',' {
                    // Safe to flush previous paragraph and start a new dialog block.
                    segments.push(buffer.clone());
                    buffer.clear();
                    buffer.push_str(&line_text);
                    dialog_state.reset();
                    dialog_state.update(&line_text);
                    continue;
                }
                // else: fall through and treat as continuation
            } else {
                // Buffer is empty or whitespace only; treat like a fresh dialog line.
                segments.push(buffer.clone());
                buffer.clear();
                buffer.push_str(&line_text);
                dialog_state.reset();
                dialog_state.update(&line_text);
                continue;
            }
        }

        // Colon + dialog continuation:
        // If previous line ends with ':' or '：' and the new line starts with a
        // dialog opener, treat it as continuation within the same paragraph.
        if let Some(last_char) = buffer_text.chars().rev().find(|c| !c.is_whitespace()) {
            if last_char == '：' || last_char == ':' {
                let after_indent = line_text.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
                if let Some(first_ch) = after_indent.chars().next() {
                    if DIALOG_OPENERS.contains(&first_ch) {
                        buffer.push_str(&line_text);
                        dialog_state.update(&line_text);
                        continue;
                    }
                }
            }
        }

        // 8) CJK punctuation-driven paragraph boundary -----------------------
        if buffer_ends_with_cjk_punct(buffer_text) && !dialog_state.is_unclosed() {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 9) Chapter-like ending lines --------------------------------------
        if is_chapter_ending_line(buffer_text) {
            segments.push(buffer.clone());
            buffer.clear();
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 10) Default soft join (no hard break detected) --------------------
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
// Constants and helpers for CJK reflow
// ---------------------------------------------------------------------------

/// CJK / mixed punctuation that usually ends a sentence or clause.
const CJK_PUNCT_END: &[char] = &[
    '。', '！', '？', '；', '：', '…', '—', '”', '」', '’', '』', '）', '】', '》', '〗', '〔',
    '〕', '〉', '］', '｝', '》', '.', '?', '!',
];

/// Closing brackets that can trail after a chapter marker (章/节/部/卷/節/回).
const CHAPTER_TRAIL_BRACKETS: &[char] = &['】', '》', '〗', '〕', '〉', '」', '』', '）'];

/// Keywords treated as headings even without "第…章".
const HEADING_KEYWORDS: &[&str] = &[
    "前言", "序章", "终章", "尾声", "后记", "番外", "尾聲", "後記",
];

/// Unified chapter markers.
const CHAPTER_MARKERS: &[char] = &['章', '节', '部', '卷', '節', '回'];

/// Disallowed suffix after chapter marker (e.g. 分卷 / 合集)
const INVALID_AFTER_MARKER: &[char] = &['分', '合'];

/// Metadata separators: fullwidth colon, ASCII colon, fullwidth ideographic space.
const METADATA_SEPARATORS: &[char] = &['：', ':', '　'];

/// Metadata keys such as 書名/作者/出版社/版權/ISBN, etc.
static METADATA_KEYS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // 1. Title / Author / Publishing
        "書名",
        "书名",
        "作者",
        "譯者",
        "译者",
        "校訂",
        "校订",
        "出版社",
        "出版時間",
        "出版时间",
        "出版日期",
        // 2. Copyright / License
        "版權",
        "版权",
        "版權頁",
        "版权页",
        "版權信息",
        "版权信息",
        // 3. Editor / Pricing
        "責任編輯",
        "责任编辑",
        "編輯",
        "编辑",
        "責編",
        "责编",
        "定價",
        "定价",
        // 4. Descriptions / Forewords (only some kept as metadata keys)
        "前言",
        "序章",
        "終章",
        "终章",
        "尾聲",
        "尾声",
        "後記",
        "后记",
        // 5. Digital Publishing (ebook platforms)
        "品牌方",
        "出品方",
        "授權方",
        "授权方",
        "電子版權",
        "数字版权",
        "掃描",
        "扫描",
        "OCR",
        // 6. CIP / Cataloging
        "CIP",
        "在版編目",
        "在版编目",
        "分類號",
        "分类号",
        "主題詞",
        "主题词",
        // 7. Publishing Cycle
        "發行日",
        "发行日",
        "初版",
        // 8. Common keys without variants
        "ISBN",
    ]
    .iter()
    .copied()
    .collect()
});

/// Dialog openers (Simplified / Traditional / JP-style).
/// Note: we intentionally do not mix these into bracket sets.
const DIALOG_OPENERS: &[char] = &['“', '‘', '「', '『', '﹁', '﹃'];

/// Bracket sets used for unmatched-bracket suppression in headings.
/// Dialog brackets are *not* included here.
const OPEN_BRACKETS: &[char] = &['（', '(', '[', '【', '《', '<', '{'];
const CLOSE_BRACKETS: &[char] = &['）', ')', ']', '】', '》', '>', '}'];

// ---------------------------------------------------------------------------
// Core helpers
// ---------------------------------------------------------------------------

/// Returns true if the given line is a compact "metadata" key-value pair,
/// such as:
///
///   書名：假面遊戲
///   作者: 東野圭吾
///   出版時間　2024-03-12
///
/// Rules:
///   - Total length <= 30 chars
///   - First separator must be one of METADATA_SEPARATORS and appear
///     between positions [1..10] (char indices)
///   - Key (before separator) must be in METADATA_KEYS
///   - There must be at least one non-space character after the separator
///   - The first non-space character after separator must not be a dialog opener
fn is_metadata_line(line: &str) -> bool {
    let s = line.trim();
    if s.is_empty() {
        return false;
    }

    if s.chars().count() > 30 {
        return false;
    }

    // Find the first separator and its char position + byte index.
    let mut char_pos = 0usize;
    let mut sep_byte_idx: Option<usize> = None;

    for (byte_idx, ch) in s.char_indices() {
        if METADATA_SEPARATORS.contains(&ch) {
            // Separator cannot be the very first character (position 0),
            // and cannot appear too far (for compact key prefixes).
            if char_pos == 0 || char_pos > 10 {
                return false;
            }
            sep_byte_idx = Some(byte_idx);
            break;
        }
        char_pos += 1;
    }

    let sep_byte_idx = match sep_byte_idx {
        Some(idx) => idx,
        None => return false,
    };

    // Extract key = s[..sep_byte_idx]
    let key = s[..sep_byte_idx].trim();
    if !METADATA_KEYS.contains(key) {
        return false;
    }

    // Find the first non-whitespace character after the separator.
    let sep_char = s[sep_byte_idx..].chars().next().unwrap();
    let mut found_next: Option<char> = None;
    let after_sep = sep_byte_idx + sep_char.len_utf8();

    for (_, ch) in s[after_sep..].char_indices() {
        if ch.is_whitespace() {
            continue;
        }
        found_next = Some(ch);
        break;
    }

    let first_after = match found_next {
        Some(ch) => ch,
        None => return false, // no value after key:sep
    };

    // If next char is a dialog opener, this is more like a dialog, not metadata.
    if DIALOG_OPENERS.contains(&first_after) {
        return false;
    }

    true
}

/// Detects visual separator / divider lines such as:
/// ──────, ======, ------, or mixed variants (e.g. ───===───).
///
/// Intended to run on a *probe* string (indentation removed). Whitespace is ignored.
/// These lines represent layout boundaries and must always force paragraph breaks.
#[inline]
pub fn is_box_drawing_line(s: &str) -> bool {
    // Equivalent to string.IsNullOrWhiteSpace(s) == true -> return false
    if s.trim().is_empty() {
        return false;
    }

    let mut total = 0usize;

    for ch in s.chars() {
        // Ignore whitespace completely (probe may still contain gaps)
        if ch.is_whitespace() {
            continue;
        }

        total += 1;

        match ch {
            // Unicode box drawing block (U+2500–U+257F)
            '\u{2500}'..='\u{257F}' => {}

            // ASCII visual separators (common in TXT / OCR)
            '-' | '=' | '_' | '~' | '～' => {}

            // Star / asterisk-based visual dividers
            '*'  // ASTERISK (U+002A)
            | '＊' // FULLWIDTH ASTERISK (U+FF0A)
            | '★' // BLACK STAR (U+2605)
            | '☆' // WHITE STAR (U+2606)
            => {}

            // Any real text → not a pure visual divider
            _ => return false,
        }
    }

    // Require minimal visual length to avoid accidental triggers
    total >= 3
}

/// Returns true if the buffer ends with a CJK punctuation character
/// (ignoring trailing whitespace).
fn buffer_ends_with_cjk_punct(s: &str) -> bool {
    if let Some(ch) = s.chars().rev().find(|c| !c.is_whitespace()) {
        CJK_PUNCT_END.contains(&ch)
    } else {
        false
    }
}

/// Returns true if the line is a page marker like "=== [3/250] ===".
fn is_page_marker(s: &str) -> bool {
    s.starts_with("=== ") && s.ends_with("===")
}

/// Heading detection:
/// - 短行（<= 50 chars）
// — - 前言 / 序章 / 终章 / 尾声 / 后记 / 番外…
// — - 第…章 / 节 / 部 / 卷 / 節 / 回（带距离约束）
fn is_title_heading_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    let char_count = s.chars().count();
    if char_count > 50 {
        return false;
    }

    /* ---------- 1) Direct keyword match ---------- */
    for &kw in HEADING_KEYWORDS {
        if s.starts_with(kw) {
            return true;
        }
    }

    /* ---------- 2) 番外 + optional suffix ---------- */
    if let Some(rest) = s.strip_prefix("番外") {
        return rest.chars().count() <= 15;
    }

    /* ---------- 3) 第…章 / 节 / 部 / 卷 / 節 / 回 ---------- */
    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        if chars[i] != '第' {
            continue;
        }

        // 第 之前 ≤ 10 chars
        if i > 10 {
            continue;
        }

        // 在「第」之後找章標記
        for j in (i + 1)..chars.len() {
            // 第 → 章距離 ≤ 5
            if j - i > 6 {
                break;
            }

            let ch = chars[j];
            if !CHAPTER_MARKERS.contains(&ch) {
                continue;
            }

            // 章標記後不能是「分 / 合」
            if let Some(next) = chars.get(j + 1) {
                if INVALID_AFTER_MARKER.contains(next) {
                    return false;
                }
            }

            // 章標記後剩餘 ≤ 20 chars
            if chars.len().saturating_sub(j + 1) <= 20 {
                return true;
            }
        }
    }

    false
}

/// Chapter-like ending: line <= 15 chars and the last non-bracket char is
/// 章/节/部/卷/節/回 (possibly followed by decorative brackets).
fn is_chapter_ending_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    if s.chars().count() > 15 {
        return false;
    }

    // Strip trailing chapter brackets like 】》〗〕〉」』）.
    let mut trimmed = s;
    loop {
        if let Some(last) = trimmed.chars().last() {
            if CHAPTER_TRAIL_BRACKETS.contains(&last) {
                let new_len = trimmed.len() - last.len_utf8();
                trimmed = &trimmed[..new_len];
                continue;
            }
        }
        break;
    }

    if let Some(last) = trimmed.chars().last() {
        CHAPTER_MARKERS.contains(&last)
    } else {
        false
    }
}

/// Returns true if the given line begins with a dialog opener,
/// ignoring both halfwidth and fullwidth indentation.
fn is_dialog_start(s: &str) -> bool {
    let trimmed = s.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
    if let Some(ch) = trimmed.chars().next() {
        DIALOG_OPENERS.contains(&ch)
    } else {
        false
    }
}

/// Heading-like heuristic for short CJK titles / emphasis lines.
///
/// Rules:
///   - Reject page markers ("=== ... ===").
///   - Reject lines that end with CJK end punctuation.
///   - Reject lines with unclosed brackets.
///   - Reject any line (short or long) that contains '，', ',' or '、'.
///   - For short lines (len <= 10):
///       * If they contain any CJK punctuation at all → not heading.
///       * Pure ASCII digits → heading (e.g. "1", "007").
///       * CJK/mixed short line (has non-ASCII, no comma) → heading.
///       * Pure ASCII short line with at least one letter → heading.
fn is_heading_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Keep page markers intact (handled separately).
    if s.starts_with("=== ") && s.ends_with("===") {
        return false;
    }

    // Reject headings with unclosed brackets: has any open bracket but no close bracket.
    let has_open = s.chars().any(|ch| OPEN_BRACKETS.contains(&ch));
    let has_close = s.chars().any(|ch| CLOSE_BRACKETS.contains(&ch));
    if has_open && !has_close {
        return false;
    }

    let len = s.chars().count();
    let max_len = if is_all_ascii(s) { 16 } else { 8 };

    // If ends with CJK end punctuation → not heading
    if let Some(last) = s.chars().last() {
        // Short circuit for item title-like: "物品准备："
        if (last == '：' || last == ':') && len < max_len {
            let body = strip_last_char(s);
            if is_all_cjk(body) {
                return true;
            }
        }

        if CJK_PUNCT_END.contains(&last) {
            return false;
        }
    }

    // NEW: reject any line that contains a comma-like delimiter.
    // Short headings should never contain "，" / "," / "、".
    if s.contains('，') || s.contains(',') || s.contains('、') {
        return false;
    }

    if len <= max_len {
        // NEW: any short line containing CJK punctuation is not heading.
        if s.chars().any(|ch| CJK_PUNCT_END.contains(&ch)) {
            return false;
        }

        let mut has_non_ascii = false;
        let mut all_ascii = true;
        let mut has_letter = false;
        let mut all_ascii_digits = true;

        for ch in s.chars() {
            if (ch as u32) > 0x7F {
                has_non_ascii = true;
                all_ascii = false;
                all_ascii_digits = false;
                continue;
            }

            if !ch.is_ascii_digit() {
                all_ascii_digits = false;
            }

            if ch.is_ascii_alphabetic() {
                has_letter = true;
            }
        }

        // Rule C: pure ASCII digits → heading (e.g. "1", "007").
        if all_ascii_digits {
            return true;
        }

        // Rule A: short CJK/mixed line (has non-ASCII) → heading.
        if has_non_ascii {
            return true;
        }

        // Rule B: short ASCII line with at least one letter → heading.
        if all_ascii && has_letter {
            return true;
        }
    }

    false
}

/// True if every char is ASCII (<= 0x7F). Empty string => true (same as your C#).
#[inline]
pub fn is_all_ascii(s: &str) -> bool {
    // fastest: ASCII is a byte property
    s.is_ascii()
}

/// True if s is non-empty and every char is an ASCII digit [0-9].
#[inline]
pub fn is_all_ascii_digits(s: &str) -> bool {
    // C# returns false for empty
    !s.is_empty() && s.bytes().all(|b| (b'0'..=b'9').contains(&b))
}

/// True if s is non-empty, contains no whitespace, and every char is CJK (BMP-focused).
/// Note: Rust `char::is_whitespace()` returns true for ideographic space U+3000 too,
/// matching your “treat common full-width space as not CJK heading content”.
#[inline]
pub fn is_all_cjk(s: &str) -> bool {
    let mut any = false;

    for ch in s.chars() {
        any = true;

        if ch.is_whitespace() {
            return false;
        }
        if !is_cjk_bmp(ch) {
            return false;
        }
    }

    any
}

/// Minimal CJK checker (BMP focused), matching your C# ranges:
/// - Extension A: U+3400..=U+4DBF
/// - Unified:     U+4E00..=U+9FFF
/// - Compat:      U+F900..=U+FAFF
#[inline]
pub fn is_cjk_bmp(ch: char) -> bool {
    let c = ch as u32;

    // CJK Unified Ideographs Extension A
    if (0x3400..=0x4DBF).contains(&c) {
        return true;
    }
    // CJK Unified Ideographs
    if (0x4E00..=0x9FFF).contains(&c) {
        return true;
    }
    // CJK Compatibility Ideographs
    (0xF900..=0xFAFF).contains(&c)
}

#[inline]
fn is_all_cjk_ignoring_ws(s: &str) -> bool {
    let mut any = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
        any = true;
        // C# logic: ASCII => not all-CJK
        if (ch as u32) <= 0x7F {
            return false;
        }
    }
    any
}

/// Check if the string contains any unclosed opening bracket.
///
/// Semantics (identical to C#):
/// - Returns false for empty string
/// - Returns true if:
///     * at least one opening bracket is found
///     * no closing bracket is found anywhere
/// - Does NOT attempt proper nesting or ordering
#[inline]
pub fn has_unclosed_bracket(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut has_open = false;
    let mut has_close = false;

    for ch in s.chars() {
        if !has_open && OPEN_BRACKETS.contains(&ch) {
            has_open = true;
        }
        if !has_close && CLOSE_BRACKETS.contains(&ch) {
            has_close = true;
        }

        if has_open && has_close {
            break;
        }
    }

    has_open && !has_close
}

// ---------------------------------------------------------------------------
// Collapse repeated segments (style-layer de-duplication)
// ---------------------------------------------------------------------------

/// Collapse style-layer repeated segments within a line.
///
/// This is a two-step process:
///
///   1. Phrase-level collapse:
///      Detect short token sequences that repeat 3+ times and collapse
///      them, e.g.:
///
///        "背负着一切的麒麟 背负着一切的麒麟 背负着一切的麒麟 背负着一切的麒麟"
///          → "背负着一切的麒麟"
///
///   2. Token-level collapse:
///      If a single token is made entirely of a repeated substring of
///      length 4..10, repeated at least 3 times, collapse that token:
///
///        "abcdabcdabcd" → "abcd"
///
/// Very short units and natural patterns such as "哈哈哈哈哈哈" are
/// intentionally left intact.
fn collapse_repeated_segments(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_owned();
    }

    // Split into tokens by whitespace; multiple spaces/tabs are collapsed.
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return line.to_owned();
    }

    // 1) Phrase-level collapse
    let phrase_collapsed = collapse_repeated_word_sequences(&parts);

    // 2) Token-level collapse
    let token_collapsed: Vec<String> = phrase_collapsed
        .into_iter()
        .map(|tok| collapse_repeated_token(&tok))
        .collect();

    token_collapsed.join(" ")
}

/// Collapse repeated *word sequences* (phrases) within a single line.
///
/// Example:
///   ["背负着一切的麒麟",
///    "背负着一切的麒麟",
///    "背负着一切的麒麟",
///    "背负着一切的麒麟"]
///
/// → ["背负着一切的麒麟"]
fn collapse_repeated_word_sequences(parts: &[&str]) -> Vec<String> {
    const MIN_REPEATS: usize = 3;
    const MAX_PHRASE_LEN: usize = 8;

    let n = parts.len();
    if n < MIN_REPEATS {
        return parts.iter().map(|s| (*s).to_owned()).collect();
    }

    for start in 0..n {
        for phrase_len in 1..=MAX_PHRASE_LEN {
            if start + phrase_len > n {
                break;
            }

            let mut count = 1;

            loop {
                let next_start = start + count * phrase_len;
                if next_start + phrase_len > n {
                    break;
                }

                let mut equal = true;
                for k in 0..phrase_len {
                    if parts[start + k] != parts[next_start + k] {
                        equal = false;
                        break;
                    }
                }

                if !equal {
                    break;
                }

                count += 1;
            }

            if count >= MIN_REPEATS {
                // Build collapsed result: [prefix] + [one phrase] + [tail]
                let mut result = Vec::with_capacity(n - (count - 1) * phrase_len);

                // prefix
                for i in 0..start {
                    result.push(parts[i].to_owned());
                }

                // one copy of the repeated phrase
                for k in 0..phrase_len {
                    result.push(parts[start + k].to_owned());
                }

                // tail
                let tail_start = start + count * phrase_len;
                for i in tail_start..n {
                    result.push(parts[i].to_owned());
                }

                return result;
            }
        }
    }

    parts.iter().map(|s| (*s).to_owned()).collect()
}

/// Collapse repeated substring patterns inside a single token.
///
/// Only applies when:
///   - token length is between 4 and 200
///   - base unit length is between 4 and 10
///   - the token is exactly N consecutive repeats of that unit, with N >= 3
///
/// Examples:
///   "abcdabcdabcd" → "abcd"
///   "第一季大结局第一季大结局第一季大结局" → "第一季大结局"
fn collapse_repeated_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    let length = chars.len();

    if length < 4 || length > 200 {
        return token.to_owned();
    }

    // Require at least 3 repeats (so unit_len <= length / 3)
    for unit_len in 4..=10 {
        if unit_len > length / 3 {
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

// ---------------------------------------------------------------------------
// Dialog state
// ---------------------------------------------------------------------------

/// Tracks unmatched dialog brackets for the current paragraph buffer.
///
/// This structure is updated incrementally as lines are appended, so
/// we do not need to rescan the entire buffer every time we want to
/// know whether we are "still inside" a dialog block.
struct DialogState {
    double_quote: i32, // “ ”
    single_quote: i32, // ‘ ’
    corner: i32,       // 「 」
    corner_bold: i32,  // 『 』
    corner_top: i32,   // ﹁ ﹂
    corner_wide: i32,  // ﹄ ﹃
}

impl DialogState {
    fn new() -> Self {
        Self {
            double_quote: 0,
            single_quote: 0,
            corner: 0,
            corner_bold: 0,
            corner_top: 0,
            corner_wide: 0,
        }
    }

    fn reset(&mut self) {
        self.double_quote = 0;
        self.single_quote = 0;
        self.corner = 0;
        self.corner_bold = 0;
        self.corner_top = 0;
        self.corner_wide = 0;
    }

    /// Incrementally update quote counters based on the provided text fragment.
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
                '﹁' => self.corner_top += 1,
                '﹂' => {
                    if self.corner_top > 0 {
                        self.corner_top -= 1;
                    }
                }
                '﹃' => self.corner_wide += 1,
                '﹄' => {
                    if self.corner_wide > 0 {
                        self.corner_wide -= 1;
                    }
                }
                _ => {}
            }
        }
    }

    /// Returns true if there are any unmatched dialog brackets.
    fn is_unclosed(&self) -> bool {
        self.double_quote > 0
            || self.single_quote > 0
            || self.corner > 0
            || self.corner_bold > 0
            || self.corner_top > 0
            || self.corner_wide > 0
    }
}

// ---------------------------------------------------------------------------
// Indent helper
// ---------------------------------------------------------------------------

/// Strip only *halfwidth* leading spaces (`' '`), but keep fullwidth spaces
/// (`\u3000`). This preserves CJK paragraph indentation while normalizing
/// Western-style indent.
fn strip_halfwidth_indent_keep_fullwidth(s: &str) -> &str {
    let mut start_byte = 0;
    for (idx, ch) in s.char_indices() {
        if ch == ' ' {
            // skip halfwidth spaces
            start_byte = idx + ch.len_utf8();
            continue;
        }
        // stop at first non-halfwidth-space (including fullwidth indent)
        break;
    }
    &s[start_byte..]
}

fn strip_last_char(s: &str) -> &str {
    match s.char_indices().last() {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Python module definition for opencc_pyo3.
/// Exposes the OpenCC class to Python.
#[pymodule]
fn opencc_pyo3(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<OpenCC>()?;
    m.add_function(wrap_pyfunction!(reflow_cjk_paragraphs, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_text, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_text_pages, m)?)?;
    m.add_function(wrap_pyfunction!(extract_pdf_pages_with_callback, m)?)?;

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

    #[test]
    fn test_reflow_drawing_box() {
        let input = "\
物品准备：
购物帐单（三叔笔记复印）：
名字 数量
──────────────
洛阳铲头 5 个

──────────────
攀山绳 200 米
──────────────
林德大号开山刀 2 把
（加厚的）
──────────────
";

        // Adjust the path if your function lives elsewhere
        // e.g. crate::reflow::reflow_cjk_paragraphs
        let output = crate::reflow_cjk_paragraphs(
            input, /* add_pdf_page_header = */ false, /* compact = */ false,
        )
        .expect("Failed to reflow CJK paragraphs");

        // Print so you can see it when running `cargo test -- --nocapture`
        println!("===== INPUT =====");
        println!("{}", input.replace('\n', "\\n\n"));

        println!("===== REFLOWED =====");
        println!("{}", output.replace('\n', "\\n\n"));

        // Keep test "real"
        assert!(!output.is_empty());
    }
}
