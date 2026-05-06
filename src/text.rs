const TAB_WIDTH: usize = 4;

/// Expand tab characters to spaces using column-aware stops.
pub fn expand_tabs(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut col = 0usize;
    for ch in s.chars() {
        if ch == '\t' {
            let spaces = TAB_WIDTH - (col % TAB_WIDTH);
            for _ in 0..spaces {
                out.push(' ');
            }
            col += spaces;
        } else {
            out.push(ch);
            col += 1;
        }
    }
    out
}

/// Apply tab expansion, width limiting, and optional word-aware wrapping.
/// All indexing is Unicode-safe (char-based, not byte-based).
pub fn process_lines(raw: &str, width: Option<usize>, wrap: bool) -> Vec<String> {
    let expanded: Vec<String> = raw.lines().map(|l| expand_tabs(l)).collect();

    let Some(w) = width else { return expanded; };

    let mut out = Vec::new();
    for line in expanded {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() <= w {
            out.push(chars.iter().collect());
            continue;
        }

        if !wrap {
            out.push(chars[..w].iter().collect());
            continue;
        }

        // Word-aware wrapping: find the last space within the first `w` chars.
        let mut remaining: &[char] = &chars;
        while remaining.len() > w {
            let slice = &remaining[..w];
            let break_at = slice.iter().rposition(|&c| c == ' ').filter(|&p| p > 0).unwrap_or(w);
            out.push(remaining[..break_at].iter().collect());
            // Skip the space we broke on, then any leading spaces on the next segment.
            let next = if break_at < remaining.len() && remaining[break_at] == ' ' {
                break_at + 1
            } else {
                break_at
            };
            let skip = remaining[next..].iter().position(|&c| c != ' ').unwrap_or(0);
            remaining = &remaining[next + skip..];
        }
        if !remaining.is_empty() {
            out.push(remaining.iter().collect());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── expand_tabs ───────────────────────────────────────────────────────────

    #[test]
    fn tab_at_start_of_line() {
        assert_eq!(expand_tabs("\thello"), "    hello");
    }

    #[test]
    fn tab_aligns_to_next_stop() {
        // "ab" is 2 chars wide; next tab stop at col 4 → 2 spaces inserted
        assert_eq!(expand_tabs("ab\tcd"), "ab  cd");
    }

    #[test]
    fn consecutive_tabs() {
        assert_eq!(expand_tabs("\t\t"), "        ");
    }

    #[test]
    fn no_tabs_unchanged() {
        assert_eq!(expand_tabs("hello world"), "hello world");
    }

    #[test]
    fn box_drawing_chars_pass_through() {
        // JetBrains Mono covers box-drawing; characters reach the font unchanged
        assert_eq!(expand_tabs("\u{2500}\u{2500}\u{2500}"), "\u{2500}\u{2500}\u{2500}");
    }

    #[test]
    fn extended_latin_passes_through() {
        assert_eq!(expand_tabs("caf\u{00e9}"), "café");
    }

    // ── process_lines: passthrough ────────────────────────────────────────────

    #[test]
    fn no_width_expands_tabs_and_returns_all_lines() {
        let out = process_lines("a\tb\nc", None, false);
        assert_eq!(out, vec!["a   b", "c"]);
    }

    #[test]
    fn short_lines_pass_through_unchanged() {
        let out = process_lines("hi\nbye", Some(10), false);
        assert_eq!(out, vec!["hi", "bye"]);
    }

    // ── process_lines: truncation ─────────────────────────────────────────────

    #[test]
    fn truncate_ascii() {
        let out = process_lines("hello world", Some(5), false);
        assert_eq!(out, vec!["hello"]);
    }

    #[test]
    fn truncate_is_unicode_safe() {
        // "café" is 4 chars (not 5 bytes); truncating at 3 must not panic
        let out = process_lines("café extra", Some(3), false);
        assert_eq!(out, vec!["caf"]);
    }

    #[test]
    fn truncate_emoji_safe() {
        // Each emoji is one Unicode scalar; char-based truncation is correct
        let out = process_lines("🌟🌟🌟🌟🌟", Some(3), false);
        assert_eq!(out, vec!["🌟🌟🌟"]);
    }

    // ── process_lines: word-wrap ──────────────────────────────────────────────

    #[test]
    fn wrap_breaks_at_last_space_in_window() {
        // width=12 fits "hello world" exactly; space at index 11 is the break point
        let out = process_lines("hello world foo", Some(12), true);
        assert_eq!(out, vec!["hello world", "foo"]);
    }

    #[test]
    fn wrap_strips_leading_spaces_on_continuation() {
        // "hello" (5) fits in width=6; space is skipped on next segment
        let out = process_lines("hello world", Some(6), true);
        assert_eq!(out, vec!["hello", "world"]);
    }

    #[test]
    fn wrap_hard_breaks_when_no_space() {
        let out = process_lines("abcdefghij", Some(4), true);
        assert_eq!(out, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn wrap_is_unicode_safe() {
        // 'é' is one char; wrapping counts chars correctly regardless of byte width
        let out = process_lines("café latte", Some(5), true);
        assert_eq!(out, vec!["café", "latte"]);
    }

    #[test]
    fn wrap_handles_multiline_input() {
        let out = process_lines("one two\nthree four five", Some(9), true);
        assert_eq!(out, vec!["one two", "three", "four five"]);
    }
}
