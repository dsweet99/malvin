const MARK_PREFIX: &str = "__MALVIN_DM_EMIT__";

#[cfg(test)]
std::thread_local! {
    static MARKS: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) fn publish_mark(body: &str) {
    let line = mark_line(body);
    #[cfg(test)]
    MARKS.with(|marks| marks.borrow_mut().push(line));
    #[cfg(not(test))]
    write_mark_line(&line);
}

#[cfg(not(test))]
fn write_mark_line(line: &str) {
    use std::io::Write;
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

pub(crate) fn mark_line(body: &str) -> String {
    format!("{MARK_PREFIX}{}", encode_hex(body.as_bytes()))
}

pub(crate) fn parse_dm_emit_mark(line: &str) -> Option<Result<String, ()>> {
    let plain = crate::ansi_strip::strip_ansi_escapes(line);
    let hex = plain.trim_ascii().strip_prefix(MARK_PREFIX)?;
    if hex.is_empty() {
        return Some(Ok(String::new()));
    }
    Some(decode_hex(hex).ok_or(()))
}

#[cfg(test)]
pub(crate) fn take_test_marks() -> Vec<String> {
    MARKS.with(|marks| std::mem::take(&mut *marks.borrow_mut()))
}

#[cfg(test)]
pub(crate) fn clear_test_marks() {
    MARKS.with(|marks| marks.borrow_mut().clear());
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let value = usize::from(*byte);
        out.push(char::from(HEX[value >> 4]));
        out.push(char::from(HEX[value & 0x0f]));
    }
    out
}

fn decode_hex(text: &str) -> Option<String> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    let raw = text.as_bytes();
    let mut bytes = Vec::with_capacity(raw.len() / 2);
    let mut index = 0;
    while index < raw.len() {
        let hi = hex_value(raw[index])?;
        let lo = hex_value(raw[index + 1])?;
        bytes.push((hi << 4) | lo);
        index += 2;
    }
    String::from_utf8(bytes).ok()
}

const fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
#[path = "dm_hook_mark_tests.rs"]
mod tests;
