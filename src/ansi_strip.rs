//! Strip ANSI SGR and related sequences for plain-text comparisons.

fn consume_csi_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) {
    for ch in chars.by_ref() {
        if matches!(ch, '\x40'..='\x7e') {
            break;
        }
    }
}

fn consume_osc_sequence(chars: &mut std::iter::Peekable<std::str::Chars>) {
    while let Some(ch) = chars.next() {
        if ch == '\x07' {
            return;
        }
        if ch == '\x1b' && chars.peek() == Some(&'\\') {
            chars.next();
            return;
        }
    }
}

#[must_use]
pub fn strip_ansi_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                consume_csi_sequence(&mut chars);
            }
            Some(']') => {
                chars.next();
                consume_osc_sequence(&mut chars);
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::strip_ansi_escapes;

    #[test]
    fn strip_ansi_removes_csi_color() {
        let s = "\x1b[31mfoo\x1b[0m";
        assert_eq!(strip_ansi_escapes(s), "foo");
    }

    #[test]
    fn strip_ansi_preserves_following_text_when_csi_does_not_end_with_m() {
        assert_eq!(strip_ansi_escapes("foo\x1b[2Jbar"), "foobar");
    }

    #[test]
    fn strip_ansi_removes_osc_set_window_title() {
        let raw = "\x1b]0;cursor-agent\x07composer-2 — default";
        assert_eq!(
            strip_ansi_escapes(raw),
            "composer-2 — default",
            "OSC title/hyperlink noise should be stripped for stable `models` parsing"
        );
    }

    #[test]
    fn strip_ansi_removes_osc_terminated_with_st() {
        let raw = "x\x1b]52;c;Z\x1b\\y";
        assert_eq!(
            strip_ansi_escapes(raw),
            "xy",
            "OSC may end with ST (ESC \\) instead of BEL"
        );
    }

}
