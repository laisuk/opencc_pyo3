//! CJK PDF Reflow Engine (pure Rust)
//!
//! Extracted from opencc_pyo3 PyO3 module. Designed to be reused by:
//! - Python bindings (thin wrapper)
//! - CLI (opencc-rs PDF / office / etc.)

use once_cell::sync::Lazy;
use pyo3::{pyfunction, PyResult};
use std::collections::HashSet;

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
pub fn reflow_cjk_paragraphs(
    text: &str,
    add_pdf_page_header: bool,
    compact: bool,
) -> PyResult<String> {
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

        // 2) Collapse style-layer repeated segments per line
        let line_text = collapse_repeated_segments(stripped_visual);

        // 3) Logical probe for heading detection (no left indent)
        let heading_probe = line_text.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');

        // 4) Empty line
        if heading_probe.trim().is_empty() {
            if !add_pdf_page_header && !buffer.is_empty() {
                // LIGHT rule: only flush on blank line if buffer ends with STRONG sentence end.
                let ends_strong = buffer
                    .chars()
                    .rev()
                    .find(|c| !c.is_whitespace())
                    .map_or(false, is_strong_sentence_end);

                if !ends_strong {
                    // Soft cross-page blank line: keep accumulating
                    continue;
                }
            }

            // End paragraph → flush buffer (do not emit empty segments)
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            continue;
        }

        // 5) Page marker lines
        if is_page_marker(heading_probe) {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(line_text.clone());
            continue;
        }

        // 6) Heading / metadata detection
        let is_title_heading = is_title_heading_line(heading_probe);
        let is_short_heading = is_heading_like(&line_text);
        let is_metadata = is_metadata_line(&line_text);

        let mut flush_buffer_and_emit_standalone = |line: &str| {
            if !buffer.is_empty() {
                segments.push(std::mem::take(&mut buffer));
                dialog_state.reset();
            }
            segments.push(line.to_owned());
        };

        if is_metadata {
            flush_buffer_and_emit_standalone(&line_text);
            continue;
        }
        if is_title_heading {
            flush_buffer_and_emit_standalone(&line_text);
            continue;
        }

        let buffer_text = buffer.as_str();

        if is_short_heading {
            let stripped = heading_probe;

            if !buffer.is_empty() {
                // let buf_text = buffer.as_str();

                if has_unclosed_bracket(buffer_text) {
                    // treat as continuation
                } else {
                    let bt = buffer_text.trim_end();
                    if let Some(last) = bt.chars().last() {
                        if last == '，' || last == ',' || last == '、' {
                            // continuation
                        } else {
                            let is_all_cjk = is_all_cjk_ignoring_ws(stripped);
                            if is_all_cjk && !CJK_PUNCT_END.contains(&last) {
                                // continuation
                            } else {
                                segments.push(std::mem::take(&mut buffer));
                                dialog_state.reset();
                                segments.push(line_text.clone());
                                continue;
                            }
                        }
                    } else {
                        segments.push(line_text.clone());
                        continue;
                    }
                }
            } else {
                segments.push(line_text.clone());
                continue;
            }
        }

        // Final strong line punct ending check for line text
        let stripped = line_text.trim_end();
        if !buffer.is_empty() {
            if let Some(last) = stripped.chars().rev().next() {
                if is_strong_sentence_end(last) {
                    buffer.push_str(&line_text);
                    segments.push(std::mem::take(&mut buffer));
                    dialog_state.reset();
                    dialog_state.update(&line_text);
                    continue;
                }
            }
        }

        // 7) Dialog detection
        let current_is_dialog_start = is_dialog_start(&line_text);

        // First line of a new paragraph
        if buffer.is_empty() {
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // If previous line ends with comma, do NOT flush even if new line starts dialog
        if current_is_dialog_start {
            let trimmed_buffer = buffer_text.trim_end();
            let last = trimmed_buffer.chars().rev().next();
            if let Some(ch) = last {
                if ch != '，' && ch != ',' && ch != '、' {
                    segments.push(std::mem::take(&mut buffer));
                    buffer.push_str(&line_text);
                    dialog_state.reset();
                    dialog_state.update(&line_text);
                    continue;
                }
            } else {
                segments.push(std::mem::take(&mut buffer));
                buffer.push_str(&line_text);
                dialog_state.reset();
                dialog_state.update(&line_text);
                continue;
            }
        }

        // Colon + dialog continuation
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

        // 8a) Strong sentence boundary (handles 。！？, OCR . / :, “.”)
        if !dialog_state.is_unclosed() && ends_with_sentence_boundary(buffer_text) {
            segments.push(std::mem::take(&mut buffer));
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 8b) Balanced CJK bracket boundary: （完）, 【番外】, 《後記》
        if !dialog_state.is_unclosed() && ends_with_cjk_bracket_boundary(buffer_text) {
            segments.push(std::mem::take(&mut buffer));
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 8c) Broad punctuation fallback
        if !dialog_state.is_unclosed() && buffer_ends_with_cjk_punct(buffer_text) {
            segments.push(std::mem::take(&mut buffer));
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 9) Chapter-like ending lines
        if is_chapter_ending_line(buffer_text) {
            segments.push(std::mem::take(&mut buffer));
            buffer.push_str(&line_text);
            dialog_state.reset();
            dialog_state.update(&line_text);
            continue;
        }

        // 10) Default soft join
        buffer.push_str(&line_text);
        dialog_state.update(&line_text);
    }

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
// Constants and helpers (copied from your original)
// ---------------------------------------------------------------------------

const CJK_PUNCT_END: &[char] = &[
    '。', '！', '？', '；', '：', '…', '—', '”', '」', '’', '』', '）', '】', '》', '〗', '〔',
    '〕', '〉', '⟩', '］', '｝', '》', '＞', '.', '?', '!',
];

const CHAPTER_TRAIL_BRACKETS: &[char] = &[
    '】', '》', '〗', '〕', '〉', '」', '』', '）', '］', '＞', '⟩',
];

const HEADING_KEYWORDS: &[&str] = &[
    "前言", "序章", "终章", "尾声", "后记", "番外", "尾聲", "後記",
];

const CHAPTER_MARKERS: &[char] = &['章', '节', '部', '卷', '節', '回'];
const INVALID_AFTER_MARKER: &[char] = &['分', '合'];
const HEADING_REJECT_PUNCT: &[char] = &['，', ',', '。', '！', '？', '；'];

const CJK_NUMERALS: &[char] = &['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

const METADATA_SEPARATORS: &[char] = &['：', ':', '・', '　'];

static METADATA_KEYS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
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
        "版權",
        "版权",
        "版權頁",
        "版权页",
        "版權信息",
        "版权信息",
        "責任編輯",
        "责任编辑",
        "編輯",
        "编辑",
        "責編",
        "责编",
        "定價",
        "定价",
        "前言",
        "序章",
        "終章",
        "终章",
        "尾聲",
        "尾声",
        "後記",
        "后记",
        "品牌方",
        "出品方",
        "授權方",
        "授权方",
        "電子版權",
        "数字版权",
        "掃描",
        "扫描",
        "OCR",
        "CIP",
        "在版編目",
        "在版编目",
        "分類號",
        "分类号",
        "主題詞",
        "主题词",
        "發行日",
        "发行日",
        "初版",
        "ISBN",
    ]
    .iter()
    .copied()
    .collect()
});

const DIALOG_OPENERS: &[char] = &['“', '‘', '「', '『', '﹁', '﹃'];

pub const DIALOG_CLOSERS: &[char] = &[
    // Standard paired closers
    '”', '’', '」', '』', '﹂', '﹄', // Occasionally seen variants / compatibility forms
    '〞', '〟',
];

#[inline]
pub fn is_dialog_opener(ch: char) -> bool {
    DIALOG_OPENERS.contains(&ch)
}

#[inline]
pub fn is_dialog_closer(ch: char) -> bool {
    DIALOG_CLOSERS.contains(&ch)
}

/// Bracket punctuations (open → close)
const BRACKET_PAIRS: &[(char, char)] = &[
    // Parentheses
    ('（', '）'),
    ('(', ')'),
    // Square brackets
    ('［', '］'),
    ('[', ']'),
    // Curly braces
    ('｛', '｝'),
    ('{', '}'),
    // Angle brackets
    ('＜', '＞'),
    ('<', '>'),
    ('⟨', '⟩'),
    ('〈', '〉'),
    // CJK brackets
    ('【', '】'),
    ('《', '》'),
    ('〔', '〕'),
    ('〖', '〗'),
];

#[inline]
fn is_bracket_opener(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(open, _)| open == ch)
}

#[inline]
fn is_bracket_closer(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(_, close)| close == ch)
}

#[inline]
fn is_matching_bracket(open: char, close: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(o, c)| o == open && c == close)
}

fn is_metadata_line(line: &str) -> bool {
    let s = line.trim();
    if s.is_empty() || s.chars().count() > 30 {
        return false;
    }

    let mut char_pos = 0usize;
    let mut sep_byte_idx: Option<usize> = None;

    for (byte_idx, ch) in s.char_indices() {
        if METADATA_SEPARATORS.contains(&ch) {
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

    let key = s[..sep_byte_idx].trim();
    if !METADATA_KEYS.contains(key) {
        return false;
    }

    let sep_char = s[sep_byte_idx..].chars().next().unwrap();
    let after_sep = sep_byte_idx + sep_char.len_utf8();

    let mut found_next: Option<char> = None;
    for (_, ch) in s[after_sep..].char_indices() {
        if ch.is_whitespace() {
            continue;
        }
        found_next = Some(ch);
        break;
    }

    let first_after = match found_next {
        Some(ch) => ch,
        None => return false,
    };

    !DIALOG_OPENERS.contains(&first_after)
}

#[inline]
pub fn is_box_drawing_line(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }

    let mut total = 0usize;

    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
        total += 1;

        match ch {
            '\u{2500}'..='\u{257F}' => {}
            '-' | '=' | '_' | '~' | '～' => {}
            '*' | '＊' | '★' | '☆' => {}
            _ => return false,
        }
    }

    total >= 3
}

fn buffer_ends_with_cjk_punct(s: &str) -> bool {
    s.chars()
        .rev()
        .find(|c| !c.is_whitespace())
        .is_some_and(|ch| CJK_PUNCT_END.contains(&ch))
}

fn is_page_marker(s: &str) -> bool {
    s.starts_with("=== ") && s.ends_with("===")
}

fn is_title_heading_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let char_count = s.chars().count();
    if char_count > 50 {
        return false;
    }

    // ❌ Reject sentence-like lines (comma, full stop, etc.)
    if s.chars().any(|c| HEADING_REJECT_PUNCT.contains(&c)) {
        return false;
    }

    for &kw in HEADING_KEYWORDS {
        if s.starts_with(kw) {
            return true;
        }
    }

    if let Some(rest) = s.strip_prefix("番外") {
        return rest.chars().count() <= 15;
    }

    // Strong heading: 卷一 / 章十
    {
        let mut it = s.chars();
        if let (Some(first), Some(second)) = (it.next(), it.next()) {
            if (first == '卷' || first == '章')
                && CJK_NUMERALS.contains(&second)
                && char_count <= 17
            {
                return true;
            }
        }
    }

    let chars: Vec<char> = s.chars().collect();

    for i in 0..chars.len() {
        if chars[i] != '第' {
            continue;
        }
        if i > 10 {
            continue;
        }

        for j in (i + 1)..chars.len() {
            if j - i > 6 {
                break;
            }
            let ch = chars[j];
            if !CHAPTER_MARKERS.contains(&ch) {
                continue;
            }

            if let Some(next) = chars.get(j + 1) {
                if INVALID_AFTER_MARKER.contains(next) {
                    return false;
                }
            }

            if chars.len().saturating_sub(j + 1) <= 20 {
                return true;
            }
        }
    }

    false
}

fn is_chapter_ending_line(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() || s.chars().count() > 15 {
        return false;
    }

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

    trimmed
        .chars()
        .last()
        .is_some_and(|last| CHAPTER_MARKERS.contains(&last))
}

fn is_dialog_start(s: &str) -> bool {
    let trimmed = s.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
    trimmed
        .chars()
        .next()
        .is_some_and(|ch| is_dialog_opener(ch))
}

fn is_heading_like(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if s.starts_with("=== ") && s.ends_with("===") {
        return false;
    }

    if has_unclosed_bracket(s) {
        return false;
    }

    // If the whole line is wrapped by a matching bracket pair, treat as heading-like.
    // Examples: （第一章）, 【序章】, 《后记》, 〈楔子〉
    if let (Some(first), Some(last)) = (s.chars().next(), s.chars().rev().next()) {
        if is_matching_bracket(first, last) {
            // Ensure some content inside brackets (not just "（）")
            let inner = s
                .strip_prefix(first)
                .and_then(|t| t.strip_suffix(last))
                .unwrap_or("");

            if !inner.trim().is_empty() && is_mostly_cjk(inner) {
                return true;
            }
        }
    }

    let len = s.chars().count();
    let max_len = if is_all_ascii(s) || is_mixed_cjk_ascii(s) {
        16
    } else {
        8
    };

    if let Some(last) = s.chars().last() {
        if (last == '：' || last == ':') && len < max_len {
            let body = strip_last_char(s);
            if is_all_cjk_no_ws(body) {
                return true;
            }
        }
        if CJK_PUNCT_END.contains(&last) {
            return false;
        }
    }

    if s.contains('，') || s.contains(',') || s.contains('、') {
        return false;
    }

    if len <= max_len {
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

        if all_ascii_digits {
            return true;
        }
        if has_non_ascii {
            return true;
        }
        if all_ascii && has_letter {
            return true;
        }
    }

    false
}

#[inline]
pub fn is_all_ascii(s: &str) -> bool {
    s.is_ascii()
}

#[inline]
pub fn is_all_cjk_no_ws(s: &str) -> bool {
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

#[inline]
pub fn is_cjk_bmp(ch: char) -> bool {
    let c = ch as u32;
    (0x3400..=0x4DBF).contains(&c)
        || (0x4E00..=0x9FFF).contains(&c)
        || (0xF900..=0xFAFF).contains(&c)
}

#[inline]
fn is_all_cjk_ignoring_ws(s: &str) -> bool {
    let mut any = false;
    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
        any = true;
        if (ch as u32) <= 0x7F {
            return false;
        }
    }
    any
}

#[inline]
pub fn is_mixed_cjk_ascii(s: &str) -> bool {
    let mut has_cjk = false;
    let mut has_ascii = false;

    for ch in s.chars() {
        // Neutral ASCII (allowed, but doesn't count as ASCII content)
        match ch {
            ' ' | '-' | '/' | ':' | '.' => continue,
            _ => {}
        }

        let u = ch as u32;

        if u <= 0x7F {
            // ASCII range
            if ch.is_ascii_alphanumeric() {
                has_ascii = true;
            } else {
                // Disallowed ASCII symbol
                return false;
            }
        }
        // Full-width digits: '０'..'９'
        else if (0xFF10..=0xFF19).contains(&u) {
            has_ascii = true;
        }
        // CJK BMP
        else if is_cjk_bmp(ch) {
            has_cjk = true;
        }
        // Anything else is invalid
        else {
            return false;
        }

        // Early exit (same as C#)
        if has_cjk && has_ascii {
            return true;
        }
    }

    false
}

#[inline]
fn is_mostly_cjk(s: &str) -> bool {
    let mut cjk = 0usize;
    let mut ascii = 0usize;

    for ch in s.chars() {
        // Neutral whitespace
        if ch.is_whitespace() {
            continue;
        }

        // Neutral digits (ASCII + FULLWIDTH)
        if is_digit_ascii_or_fullwidth(ch) {
            continue;
        }

        if is_cjk_bmp(ch) {
            cjk += 1;
            continue;
        }

        // Count ASCII letters only; ASCII punctuation is neutral
        if ch <= '\u{7F}' && ch.is_ascii_alphabetic() {
            ascii += 1;
        }
    }

    cjk > 0 && cjk >= ascii
}

#[inline]
fn is_digit_ascii_or_fullwidth(ch: char) -> bool {
    // ASCII digits
    if ch >= '0' && ch <= '9' {
        return true;
    }
    // FULLWIDTH digits
    ch >= '０' && ch <= '９'
}

#[inline]
pub fn has_unclosed_bracket(s: &str) -> bool {
    let mut has_open = false;
    let mut has_close = false;

    for ch in s.chars() {
        has_open |= is_bracket_opener(ch);
        has_close |= is_bracket_closer(ch);
        if has_open && has_close {
            break;
        }
    }

    has_open && !has_close
}

// ------ Sentence Boundary start ------ //

/// Level-2 normalized sentence boundary detection, INCLUDING OCR artifacts:
/// - ASCII '.' / ':' at end-of-line in mostly-CJK text (treat like '。' / '：')
/// - ASCII '.' before closers: `“.”` / `.」` / `.）` (treat like '。' before quote/bracket)
///
/// Assumptions (already in your codebase):
/// - `is_mostly_cjk(s: &str) -> bool`
/// - `is_dialog_closer(ch: char) -> bool`
/// - `is_cjk(ch: char) -> bool`
pub fn ends_with_sentence_boundary(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }

    let last_non_ws = match find_last_non_whitespace_char_index(s) {
        Some(i) => i,
        None => return false,
    };

    let last = nth_char(s, last_non_ws);

    // 1) Strong sentence enders.
    if is_strong_sentence_end(last) {
        return true;
    }

    // 2) Level-2 ALSO accepts OCR '.' / ':' at line end (mostly-CJK).
    //    (This is what your C# "case 2 / case 3" does in level>=3, but you want it in level=2.)
    if (last == '.' || last == ':') && is_ocr_cjk_ascii_punct_at_line_end(s, last_non_ws) {
        return true;
    }

    // 3) Quote closers after strong end, plus OCR artifact `.“”` / `.」` / `.）`.
    if is_quote_closer(last) {
        if let Some(prev_non_ws) = find_prev_non_whitespace_char_index(s, last_non_ws) {
            let prev = nth_char(s, prev_non_ws);

            // Strong end immediately before quote closer.
            if is_strong_sentence_end(prev) {
                return true;
            }

            // OCR artifact: ASCII '.' before closers.
            if prev == '.' && is_ocr_cjk_ascii_punct_before_closers(s, prev_non_ws) {
                return true;
            }

            // (Optional) If also want OCR ':' before closers like `“:”`, enable this:
            // if prev == ':' && is_ocr_cjk_ascii_punct_before_closers(s, prev_non_ws) { return true; }
        }
    }

    // 4) Bracket closers with mostly CJK.
    if is_bracket_closer(last) && last_non_ws > 0 && is_mostly_cjk(s) {
        return true;
    }

    // 5) Ellipsis as weak boundary.
    if ends_with_ellipsis(s) {
        return true;
    }

    false
}

#[inline]
fn nth_char(s: &str, idx: usize) -> char {
    s.chars().nth(idx).unwrap_or('\0')
}

#[inline]
fn is_quote_closer(ch: char) -> bool {
    is_dialog_closer(ch)
}

#[inline]
fn is_strong_sentence_end(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '!' | '?')
}

/// Last non-whitespace char index (char index).
fn find_last_non_whitespace_char_index(s: &str) -> Option<usize> {
    let mut char_pos = s.chars().count();

    for ch in s.chars().rev() {
        char_pos -= 1;
        if !ch.is_whitespace() {
            return Some(char_pos);
        }
    }
    None
}

/// Previous non-whitespace char index strictly before `end_exclusive` (char index).
fn find_prev_non_whitespace_char_index(s: &str, end_exclusive: usize) -> Option<usize> {
    let mut char_pos = end_exclusive;

    // IMPORTANT: reverse AFTER take() is unsafe on some toolchains,
    // so we manually limit using a counter instead.
    for ch in s.chars().rev() {
        if char_pos == 0 {
            break;
        }
        char_pos -= 1;
        if !ch.is_whitespace() {
            return Some(char_pos);
        }
    }
    None
}

/// Strict OCR: punct itself is at end-of-line (only whitespace after it),
/// and preceded by CJK in a mostly-CJK line.
fn is_ocr_cjk_ascii_punct_at_line_end(s: &str, punct_index: usize) -> bool {
    if punct_index == 0 {
        return false;
    }
    if !is_at_line_end_ignoring_whitespace(s, punct_index) {
        return false;
    }
    let prev = nth_char(s, punct_index - 1);
    is_cjk_bmp(prev) && is_mostly_cjk(s)
}

/// Relaxed OCR: after punct, allow only whitespace and closers (quote/bracket).
/// This enables `“.”` / `.」` / `.）` to count as sentence boundary.
fn is_ocr_cjk_ascii_punct_before_closers(s: &str, punct_index: usize) -> bool {
    if punct_index == 0 {
        return false;
    }
    if !is_at_end_allowing_closers(s, punct_index) {
        return false;
    }
    let prev = nth_char(s, punct_index - 1);
    is_cjk_bmp(prev) && is_mostly_cjk(s)
}

fn is_at_line_end_ignoring_whitespace(s: &str, index: usize) -> bool {
    s.chars().skip(index + 1).all(|c| c.is_whitespace())
}

fn is_at_end_allowing_closers(s: &str, index: usize) -> bool {
    for ch in s.chars().skip(index + 1) {
        if ch.is_whitespace() {
            continue;
        }
        if is_quote_closer(ch) || is_bracket_closer(ch) {
            continue;
        }
        return false;
    }
    true
}

fn ends_with_ellipsis(s: &str) -> bool {
    let t = s.trim_end();
    t.ends_with('…') || t.ends_with("……") || t.ends_with("...") || t.ends_with("..")
}

// ------ Sentence Boundary end ------ //

// ------ Bracket Boundary start ------ //

/// Returns true if the string ends with a balanced CJK-style bracket boundary,
/// e.g. （完）, 【番外】, 《後記》
///
/// Conditions:
/// 1) Trimmed string must start with an opener and end with its matching closer
/// 2) String must be mostly CJK (filters "(test)", "[1.2]")
/// 3) The bracket type must be balanced inside the string
pub fn ends_with_cjk_bracket_boundary(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }

    // Need at least two chars: open + close
    let mut chars = t.chars();
    let open = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    let close = match t.chars().rev().next() {
        Some(c) => c,
        None => return false,
    };

    // 1) Must be a known matching pair
    if !is_matching_bracket(open, close) {
        return false;
    }

    // 2) Avoid Latin cases like "(test)" or "[1.2]"
    if !is_mostly_cjk(t) {
        return false;
    }

    // 3) Ensure this bracket type is balanced inside the string
    is_bracket_type_balanced(t, open, close)
}

#[inline]
fn is_bracket_type_balanced(s: &str, open: char, close: char) -> bool {
    let mut depth: i32 = 0;

    for ch in s.chars() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth < 0 {
                // Closing before opening → malformed OCR
                return false;
            }
        }
    }

    depth == 0
}

// ------ Bracket Boundary end ------ //
fn collapse_repeated_segments(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return line.to_owned();
    }

    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return line.to_owned();
    }

    let phrase_collapsed = collapse_repeated_word_sequences(&parts);
    let token_collapsed: Vec<String> = phrase_collapsed
        .into_iter()
        .map(|tok| collapse_repeated_token(&tok))
        .collect();

    token_collapsed.join(" ")
}

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
                let mut result = Vec::with_capacity(n - (count - 1) * phrase_len);
                for i in 0..start {
                    result.push(parts[i].to_owned());
                }
                for k in 0..phrase_len {
                    result.push(parts[start + k].to_owned());
                }
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

fn collapse_repeated_token(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    let length = chars.len();

    if length < 4 || length > 200 {
        return token.to_owned();
    }

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

struct DialogState {
    double_quote: i32,
    single_quote: i32,
    corner: i32,
    corner_bold: i32,
    corner_top: i32,
    corner_wide: i32,
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

    fn update(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '“' => self.double_quote += 1,
                '”' => self.double_quote = (self.double_quote - 1).max(0),
                '‘' => self.single_quote += 1,
                '’' => self.single_quote = (self.single_quote - 1).max(0),
                '「' => self.corner += 1,
                '」' => self.corner = (self.corner - 1).max(0),
                '『' => self.corner_bold += 1,
                '』' => self.corner_bold = (self.corner_bold - 1).max(0),
                '﹁' => self.corner_top += 1,
                '﹂' => self.corner_top = (self.corner_top - 1).max(0),
                '﹃' => self.corner_wide += 1,
                '﹄' => self.corner_wide = (self.corner_wide - 1).max(0),
                _ => {}
            }
        }
    }

    fn is_unclosed(&self) -> bool {
        self.double_quote > 0
            || self.single_quote > 0
            || self.corner > 0
            || self.corner_bold > 0
            || self.corner_top > 0
            || self.corner_wide > 0
    }
}

fn strip_halfwidth_indent_keep_fullwidth(s: &str) -> &str {
    let mut start_byte = 0;
    for (idx, ch) in s.char_indices() {
        if ch == ' ' {
            start_byte = idx + ch.len_utf8();
            continue;
        }
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
