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

