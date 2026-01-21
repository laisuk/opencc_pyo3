use crate::punct_sets::*;

#[inline]
pub fn is_all_ascii(s: &str) -> bool {
    s.is_ascii()
}

#[inline]
pub fn is_all_cjk(s: &str, allow_whitespace: bool) -> bool {
    let mut seen = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !allow_whitespace {
                return false;
            }
            continue;
        }

        seen = true;

        if !is_cjk_bmp(ch) {
            return false;
        }
    }

    // false for empty / whitespace-only
    seen
}

#[inline]
pub fn is_all_cjk_ignoring_ws(s: &str) -> bool {
    is_all_cjk(s, true)
}

#[inline]
pub fn is_all_cjk_no_ws(s: &str) -> bool {
    is_all_cjk(s, false)
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

/// “Mostly CJK” heuristic used by a few boundary rules.
///
/// - Counts CJK letters as CJK.
/// - Counts ASCII alphabetic letters as ASCII.
/// - Treats digits and whitespace as neutral.
#[inline]
pub fn is_mostly_cjk(s: &str) -> bool {
    let mut cjk = 0usize;
    let mut ascii = 0usize;

    for ch in s.chars() {
        if ch.is_whitespace() {
            continue;
        }
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

/// Minimal CJK checker (BMP focused).
/// Designed for heading / structure heuristics, not full Unicode linguistics.
#[inline]
pub fn is_cjk_bmp(ch: char) -> bool {
    let c = ch as u32;
    (0x3400..=0x4DBF).contains(&c)
        || (0x4E00..=0x9FFF).contains(&c)
        || (0xF900..=0xFAFF).contains(&c)
}

#[inline(always)]
pub fn contains_any_cjk_str(s: &str) -> bool {
    s.chars().any(is_cjk_bmp)
}

#[inline]
pub fn is_digit_ascii_or_fullwidth(ch: char) -> bool {
    // ASCII digits
    if ('0'..='9').contains(&ch) {
        return true;
    }
    // FULLWIDTH digits
    ('０'..='９').contains(&ch)
}

pub fn strip_halfwidth_indent_keep_fullwidth(s: &str) -> &str {
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

pub fn strip_last_char(s: &str) -> &str {
    match s.char_indices().last() {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

// ------ Sentence Boundary start ------ //

/// Level-2 normalized sentence boundary detection.
///
/// Includes OCR artifacts (ASCII '.' / ':'), but **does not** treat a bare
/// bracket closer as a sentence boundary (that causes false flushes like "（亦作肥）").
pub fn ends_with_sentence_boundary(s: &str) -> bool {
    if s.trim().is_empty() {
        return false;
    }

    // Need index only for OCR rules; grab last + prev with byte indices.
    let Some(((last_i, last), (prev_i, prev))) = last_two_non_whitespace_idx(s) else {
        // < 2 non-whitespace chars; still may match strong end on the single char
        return last_non_whitespace(s).map_or(false, is_strong_sentence_end);
    };

    // 1) Strong sentence enders.
    if is_strong_sentence_end(last) {
        return true;
    }

    // 2) OCR '.' / ':' at line end (mostly-CJK).
    if (last == '.' || last == ':') && is_ocr_cjk_ascii_punct_at_line_end(s, last_i) {
        return true;
    }

    // 3) Quote closers + Allowed postfix closer after strong end,
    //    plus OCR artifact `.“”` / `.」` / `.）`.
    if is_dialog_closer(last) || is_allowed_postfix_closer(last) {
        if is_strong_sentence_end(prev) {
            return true;
        }

        if prev == '.' && is_ocr_cjk_ascii_punct_before_closers(s, prev_i) {
            return true;
        }
    }

    // 4) Full-width colon as a weak boundary (common: "他说：" then dialog next line)
    if is_colon_like(last) && is_mostly_cjk(s) {
        return true;
    }

    // 5) Ellipsis as weak boundary.
    if ends_with_ellipsis(s) {
        return true;
    }

    false
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

#[inline]
fn nth_char(s: &str, idx: usize) -> char {
    s.chars().nth(idx).unwrap_or('\0')
}

fn is_at_line_end_ignoring_whitespace(s: &str, index: usize) -> bool {
    s.chars().skip(index + 1).all(|c| c.is_whitespace())
}

fn is_at_end_allowing_closers(s: &str, index: usize) -> bool {
    for ch in s.chars().skip(index + 1) {
        if ch.is_whitespace() {
            continue;
        }
        if is_dialog_closer(ch) || is_bracket_closer(ch) {
            continue;
        }
        return false;
    }
    true
}

// ------ Sentence Boundary end ------

// ------ Bracket Boundary start ------

/// Returns true if the string ends with a balanced CJK-style bracket boundary,
/// e.g. （完）, 【番外】, 《後記》.
#[inline(always)]
pub fn ends_with_cjk_bracket_boundary(s: &str) -> bool {
    // Equivalent to string.IsNullOrWhiteSpace
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Need at least open+close
    let mut it = s.chars();
    let Some(open) = it.next() else {
        return false;
    };

    let Some(close) = s.chars().rev().find(|c| !c.is_whitespace()) else {
        return false;
    };

    // If trimming removed whitespace, `close` is correct; still ensure len>=2 chars
    if s.chars().count() < 2 {
        return false;
    }

    // 1) Must be one of our known pairs.
    if !is_matching_bracket(open, close) {
        return false;
    }

    // Inner content (exclude the outer bracket pair)
    // We need to slice by byte indices safely (open is first char, close is last non-ws char).
    // Since we already trimmed `s`, close is the last char of `s`.
    let inner = match slice_inner_without_outer_pair(s) {
        Some(inner) => inner.trim(),
        None => return false,
    };

    if inner.is_empty() {
        return false;
    }

    // 2) Must be mostly CJK (reject "(test)", "[1.2]", etc.)
    if !is_mostly_cjk(inner) {
        return false;
    }

    // ASCII bracket pairs are suspicious → require at least one CJK inside
    if (open == '(' || open == '[') && !contains_any_cjk_str(inner) {
        return false;
    }

    // 3) Ensure this bracket type is balanced inside the text
    //    (prevents malformed OCR / premature close)
    is_bracket_type_balanced_str(s, open)
}

/// Returns the substring excluding the first char and the last char of `s`.
/// Precondition: `s` is already trimmed and has at least 2 chars.
#[inline(always)]
fn slice_inner_without_outer_pair(s: &str) -> Option<&str> {
    // byte index after first char
    let mut iter = s.char_indices();
    let (_, first_ch) = iter.next()?;
    let after_first = first_ch.len_utf8();

    // byte index of last char start
    let (last_start, _) = s.char_indices().rev().next()?;

    if after_first > last_start {
        return None;
    }

    Some(&s[after_first..last_start])
}

#[inline(always)]
fn is_bracket_type_balanced_str(s: &str, open: char) -> bool {
    let Some(close) = try_get_matching_closer(open) else {
        // If we don't recognize the opener, treat as "balanced" (same as C# returning true)
        return true;
    };

    let mut depth: i32 = 0;

    for ch in s.chars() {
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
    }

    depth == 0
}

// ------ Bracket Boundary end ------
