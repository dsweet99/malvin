use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const TAIL_BYTES: u64 = 8_192;

pub(super) fn read(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    if len == 0 {
        return None;
    }
    let start = len.saturating_sub(TAIL_BYTES);
    let origin = if start == 0 { 0 } else { start - 1 };
    file.seek(SeekFrom::Start(origin)).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(owned_tail(&bytes, start))
}

fn owned_tail(bytes: &[u8], start: u64) -> String {
    let window = if start == 0 { bytes } else { aligned(bytes) };
    String::from_utf8_lossy(window).into_owned()
}

fn aligned(bytes: &[u8]) -> &[u8] {
    let Some((&marker, window)) = bytes.split_first() else {
        return bytes;
    };
    if marker == b'\n' {
        window
    } else {
        after_partial(window)
    }
}

fn after_partial(window: &[u8]) -> &[u8] {
    let Some(end) = window.iter().position(|byte| *byte == b'\n') else {
        return window;
    };
    let rest = &window[end + 1..];
    if rest.iter().any(|byte| !byte.is_ascii_whitespace()) {
        rest
    } else {
        window
    }
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn empty_stdout_has_no_tail() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stdout.log");
        std::fs::write(&path, "").unwrap();
        assert_eq!(read(&path), None);
    }

    #[test]
    fn short_stdout_keeps_every_line() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stdout.log");
        std::fs::write(&path, "HEAD\nTAIL\n").unwrap();
        assert_eq!(read(&path).as_deref(), Some("HEAD\nTAIL\n"));
    }
}
