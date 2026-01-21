//! Punctuation sets + helpers used by the PDF CJK reflow engine.
//!
//! Design goals
//! - Centralize punctuation / bracket / quote definitions.
//! - Keep “structure-safety” helpers (unclosed brackets, sentence boundaries)
//!   together so the reflow loop stays readable.
//!
//! NOTE: These helpers are intentionally *pessimistic* in a few places.
//! They are used for layout/reflow safety, not for perfect proofreading.

use once_cell::sync::Lazy;
use smallvec::SmallVec;
use std::collections::HashSet;

/// Broad CJK punctuation that can appear at the end of a logical unit.
///
/// Important: this set is used for *heading heuristics* and other “loose” checks.
/// It should NOT be used as a sentence-final strong boundary signal.
pub const CJK_PUNCT_END: &[char] = &[
    '。', '！', '？', '；', '：', '…', '—', '”', '」', '’', '』', '）', '】', '》', '〗', '〔',
    '〕', '〉', '⟩', '］', '｝', '》', '＞', '.', '?', '!',
];

#[inline]
pub fn is_clause_or_end_punct(ch: char) -> bool {
    CJK_PUNCT_END.contains(&ch)
}

/// Trailing brackets that may appear after a chapter marker, e.g. "第十章】".
pub const CHAPTER_TRAIL_BRACKETS: &[char] = &[
    '】', '》', '〗', '〕', '〉', '」', '』', '）', '］', '＞', '⟩',
];

pub const CHAPTER_MARKERS: &[char] = &['章', '节', '部', '卷', '節', '回'];
pub const INVALID_AFTER_MARKER: &[char] = &['分', '合'];
pub const HEADING_REJECT_PUNCT: &[char] = &['，', ',', '。', '！', '？', '；'];

pub const CJK_NUMERALS: &[char] = &['一', '二', '三', '四', '五', '六', '七', '八', '九', '十'];

pub const METADATA_SEPARATORS: &[char] = &['：', ':', '・', '　'];

pub static METADATA_KEYS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
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

pub const DIALOG_OPENERS: &[char] = &['“', '‘', '「', '『', '﹁', '﹃'];

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
pub const BRACKET_PAIRS: &[(char, char)] = &[
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
pub fn is_bracket_opener(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(open, _)| open == ch)
}

#[inline]
pub fn is_bracket_closer(ch: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(_, close)| close == ch)
}

/// Try to get the matching closing bracket for an opening bracket.
/// Returns `Some(close)` if the opener is known, otherwise `None`.
#[inline(always)]
pub fn try_get_matching_closer(open: char) -> Option<char> {
    BRACKET_PAIRS
        .iter()
        .find_map(|&(o, c)| if o == open { Some(c) } else { None })
}

#[inline]
pub fn is_allowed_postfix_closer(ch: char) -> bool {
    matches!(ch, '）' | ')')
}

#[inline]
pub fn ends_with_allowed_postfix_closer(s: &str) -> bool {
    // Trim only trailing whitespace (no allocation)
    let s = s.trim_end();

    if s.is_empty() {
        return false;
    }

    // Last non-whitespace character
    s.chars()
        .rev()
        .next()
        .map_or(false, is_allowed_postfix_closer)
}

#[inline]
pub fn is_matching_bracket(open: char, close: char) -> bool {
    BRACKET_PAIRS.iter().any(|&(o, c)| o == open && c == close)
}

#[allow(dead_code)]
#[inline(always)]
pub fn is_wrapped_by_matching_bracket(s: &str, last_non_ws: char, min_len: usize) -> bool {
    // min_len=3 means at least: open + 1 char + close
    let mut it = s.chars();
    match it.next() {
        Some(open) => s.chars().count() >= min_len && is_matching_bracket(open, last_non_ws),
        None => false,
    }
}

#[inline]
pub fn is_strong_sentence_end(ch: char) -> bool {
    matches!(ch, '。' | '！' | '？' | '!' | '?')
}

#[inline]
pub fn is_comma_like(ch: char) -> bool {
    matches!(ch, '，' | ',' | '、')
}

#[inline(always)]
pub fn contains_any_comma_like(s: &str) -> bool {
    s.chars().any(is_comma_like)
}

#[inline]
pub fn is_colon_like(ch: char) -> bool {
    matches!(ch, '：' | ':')
}

#[inline]
pub fn ends_with_colon_like(s: &str) -> bool {
    let t = s.trim_end();
    t.ends_with('：') || t.ends_with(":")
}

#[inline]
pub fn ends_with_ellipsis(s: &str) -> bool {
    let t = s.trim_end();
    t.ends_with('…') || t.ends_with("……") || t.ends_with("...") || t.ends_with("..")
}

/// Last non-whitespace char index (char index).
#[allow(dead_code)]
pub fn find_last_non_whitespace_char_index(s: &str) -> Option<usize> {
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
#[allow(dead_code)]
pub fn find_prev_non_whitespace_char_index(s: &str, end_exclusive: usize) -> Option<usize> {
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

#[inline]
pub fn last_non_whitespace(s: &str) -> Option<char> {
    s.chars().rev().find(|c| !c.is_whitespace())
}

/// Returns (byte_index, char) of the last non-whitespace char.
#[allow(dead_code)]
#[inline]
pub fn last_non_whitespace_idx(s: &str) -> Option<(usize, char)> {
    s.char_indices().rev().find(|(_, c)| !c.is_whitespace())
}

/// Returns (last, prev) non-whitespace chars (no indices).
#[inline]
pub fn last_two_non_whitespace(s: &str) -> Option<(char, char)> {
    let mut it = s.chars().rev().filter(|c| !c.is_whitespace());
    let last = it.next()?;
    let prev = it.next()?;
    Some((last, prev))
}

/// Returns ((last_i,last),(prev_i,prev)) in byte indices.
#[inline]
pub fn last_two_non_whitespace_idx(s: &str) -> Option<((usize, char), (usize, char))> {
    let mut it = s.char_indices().rev().filter(|(_, c)| !c.is_whitespace());

    let last = it.next()?;
    let prev = it.next()?;
    Some((last, prev))
}

/// Cross-page / soft-wrap safety:
/// If the previous buffer is inside an unclosed bracket like
/// "（......" ... "...。）", never flush on blank lines / weak boundaries.
///
/// NOTE: intentionally pessimistic.
/// - Any stray closer is treated as unsafe.
/// - Any mismatch is treated as unsafe.
#[inline]
pub fn has_unclosed_bracket(s: &str) -> bool {
    let mut stack: SmallVec<[char; 4]> = SmallVec::new();
    let mut seen_bracket = false;

    for ch in s.chars() {
        if is_bracket_opener(ch) {
            seen_bracket = true;
            stack.push(ch);
            continue;
        }

        if is_bracket_closer(ch) {
            seen_bracket = true;

            // STRICT: stray closer = unsafe
            let open = match stack.pop() {
                Some(o) => o,
                None => return true,
            };

            if !is_matching_bracket(open, ch) {
                return true;
            }
        }
    }

    seen_bracket && !stack.is_empty()
}

#[inline]
pub fn is_visual_divider_line(s: &str) -> bool {
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

pub fn begins_with_dialog_opener(s: &str) -> bool {
    let trimmed = s.trim_start_matches(|ch| ch == ' ' || ch == '\u{3000}');
    trimmed
        .chars()
        .next()
        .is_some_and(|ch| is_dialog_opener(ch))
}
