//! CJK PDF Reflow Engine (pure Rust)
//!
//! Extracted from opencc_pyo3 PyO3 module. Designed to be reused by:
//! - Python bindings (thin wrapper)
//! - CLI (opencc-rs PDF / office / etc.)

use crate::cjk_text::*;
use crate::punct_sets::*;
use pyo3::{pyfunction, PyResult};

/// Reflow CJK paragraphs from PDF-extracted text.
///
/// Mirrors the behavior of the original `reflow_cjk_paragraphs()` PyO3 function.
/// - Normalizes CRLF/CR to LF
/// - Merges artificial line breaks
/// - Preserves headings / metadata / page markers / dialog structure
///
/// `add_pdf_page_header`:
/// - If false, skips page-break-like blank lines not preceded by CJK punctuation.
///
/// `compact`:
/// - If true, join paragraphs with "\n"
/// - If false, join paragraphs with "\n\n"
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
        if is_visual_divider_line(probe) {
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
                let buffer_text = buffer.as_str();
                // NEW: If dialog is unclosed, always treat blank line as soft (cross-page artifact).
                // Never flush mid-dialog just because we saw a blank line.
                // Any unclosed structural enclosure (dialog OR brackets) suppresses blank-line flush.
                // Chinese prose may span multiple paragraphs inside （……） or “……”.
                if dialog_state.is_unclosed() || has_unclosed_bracket(buffer_text) {
                    continue;
                }

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
        let buffer_has_unclosed_bracket = has_unclosed_bracket(buffer_text);

        if is_short_heading {
            let stripped = heading_probe;

            // same as C# currentLooksLikeContinuationMarker
            let is_all_cjk = is_all_cjk_ignoring_ws(stripped);
            let current_looks_like_cont_marker = is_all_cjk
                || ends_with_colon_like(stripped)
                || ends_with_allowed_postfix_closer(stripped);

            let split_as_heading: bool;

            if buffer.is_empty() {
                // Start of document / just flushed
                split_as_heading = true;
            } else if buffer_has_unclosed_bracket {
                // Unsafe previous paragraph → must be continuation
                split_as_heading = false;
            } else {
                // Buffer exists, bracket is safe; inspect previous ending punctuation
                let bt = buffer_text.trim_end();

                if let Some(last) = bt.chars().last() {
                    let prev_ends_with_comma_like = is_comma_like(last);

                    // same idea as IsClauseOrEndPunct(last)
                    // (The code already uses CJK_PUNCT_END for this)
                    let prev_ends_with_sentence_punct = CJK_PUNCT_END.contains(&last);

                    // Comma-ending → continuation
                    if prev_ends_with_comma_like {
                        split_as_heading = false;
                    }
                    // All-CJK / colon-like / postfix-closer “heading-ish line”
                    // + previous not ended → continuation
                    else if current_looks_like_cont_marker && !prev_ends_with_sentence_punct {
                        split_as_heading = false;
                    } else {
                        split_as_heading = true;
                    }
                } else {
                    // Buffer is whitespace-only → treat like empty
                    split_as_heading = true;
                }
            }

            if split_as_heading {
                // If we have a real previous paragraph, flush it first
                if !buffer.is_empty() {
                    segments.push(std::mem::take(&mut buffer));
                    dialog_state.reset();
                }

                // Current line becomes a standalone heading
                segments.push(line_text.clone());
                continue;
            }

            // else: fall through → normal merge/continuation logic below
        }

        // Final strong line punct ending check for line text
        let stripped = line_text.trim_end();
        if !buffer.is_empty() && !dialog_state.is_unclosed() && !buffer_has_unclosed_bracket {
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
        let current_is_dialog_start = begins_with_dialog_opener(&line_text);

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
                if !is_comma_like(ch) && !is_cjk_bmp(ch) {
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

        // 🔸 9b) Dialog end line: ends with dialog closer.
        // Flush when the char before closer is strong end,
        // and bracket safety is satisfied (with a narrow typo override).
        if let Some((last_ch, prev_ch)) = last_two_non_whitespace(stripped) {
            if is_dialog_closer(last_ch) {
                // Check punctuation right before the closer (e.g., “？” / “。”)
                let punct_before_closer_is_strong = is_clause_or_end_punct(prev_ch);

                // Snapshot bracket safety BEFORE appending current line
                let buffer_has_bracket_issue = buffer_has_unclosed_bracket;
                let line_has_bracket_issue = has_unclosed_bracket(stripped);

                // Append current line into buffer, then update dialog state
                buffer.push_str(stripped);
                dialog_state.update(stripped);

                // Allow flush if:
                // - dialog is closed after this line
                // - punctuation before closer is a strong end
                // - and either:
                //     (a) buffer has no bracket issue, OR
                //     (b) buffer has bracket issue but this line itself is the culprit (OCR/typo),
                //         so allow a dialog-end flush anyway.
                if !dialog_state.is_unclosed()
                    && punct_before_closer_is_strong
                    && (!buffer_has_bracket_issue || line_has_bracket_issue)
                {
                    segments.push(std::mem::take(&mut buffer));
                    dialog_state.reset();
                }

                continue;
            }
        }

        // Colon + dialog continuation
        // if let Some(last_char) = buffer_text.chars().rev().find(|c| !c.is_whitespace()) {
        //     if is_colon_like(last_char) {
        //         let after_indent = line_text.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
        //         if let Some(first_ch) = after_indent.chars().next() {
        //             if DIALOG_OPENERS.contains(&first_ch) {
        //                 buffer.push_str(&line_text);
        //                 dialog_state.update(&line_text);
        //                 continue;
        //             }
        //         }
        //     }
        // }

        // 8a) Strong sentence boundary (handles 。！？, OCR . / :, “.”)
        if !dialog_state.is_unclosed()
            && ends_with_sentence_boundary(buffer_text)
            && !buffer_has_unclosed_bracket
        {
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
        // if !dialog_state.is_unclosed() && buffer_ends_with_cjk_punct(buffer_text) {
        //     segments.push(std::mem::take(&mut buffer));
        //     buffer.push_str(&line_text);
        //     dialog_state.reset();
        //     dialog_state.update(&line_text);
        //     continue;
        // }

        // 9) Chapter-like ending lines
        if !dialog_state.is_unclosed() && is_chapter_ending_line(buffer_text) {
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
// Constants and helpers
// ---------------------------------------------------------------------------

const HEADING_KEYWORDS: &[&str] = &[
    "前言", "序章", "终章", "尾声", "后记", "番外", "尾聲", "後記",
];

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

#[allow(dead_code)]
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
        if is_colon_like(last) && len < max_len {
            let body = strip_last_char(s);
            if is_all_cjk_no_ws(body) {
                return true;
            }
        }
        if is_allowed_postfix_closer(last) {
            if !contains_any_comma_like(s) {
                return true;
            }
        }
        if CJK_PUNCT_END.contains(&last) {
            return false;
        }
    }

    if contains_any_comma_like(s) {
        return false;
    }

    if len <= max_len {
        if s.chars().any(|ch| CJK_PUNCT_END.contains(&ch)) {
            return false;
        }

        let mut has_non_ascii = false;
        let mut all_ascii = true;
        let mut has_letter = false;
        let mut all_digits = true; // ASCII or full-width digits, ignoring whitespace
        let mut has_non_ws = false; // reject whitespace-only

        for ch in s.chars() {
            // whitespace is neutral
            if ch.is_whitespace() {
                continue;
            }

            has_non_ws = true;

            if !is_digit_ascii_or_fullwidth(ch) {
                all_digits = false;
            }

            if (ch as u32) > 0x7F {
                has_non_ascii = true;
                all_ascii = false;
            } else if ch.is_ascii_alphabetic() {
                has_letter = true;
            }
        }

        // digits-only (ASCII or full-width), but NOT whitespace-only
        if has_non_ws && all_digits {
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

// NOTE: punctuation / bracket / boundary helpers live in `punct_sets.rs`.
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
