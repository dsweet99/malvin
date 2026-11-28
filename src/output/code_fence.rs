#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodeFence {
    open: Option<(u8, usize)>,
}

impl CodeFence {
    #[must_use]
    pub const fn new() -> Self {
        Self { open: None }
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn observe(&mut self, line: &str) -> bool {
        let Some((mark, len, rest)) = fence_run(line) else {
            return false;
        };
        match self.open {
            None if mark == b'`' && rest.contains('`') => false,
            None => {
                self.open = Some((mark, len));
                true
            }
            Some((open_mark, open_len))
                if mark == open_mark && len >= open_len && rest.trim().is_empty() =>
            {
                self.open = None;
                true
            }
            Some(_) => false,
        }
    }
}

fn fence_run(line: &str) -> Option<(u8, usize, &str)> {
    let trimmed = line.trim_start();
    let mark = *trimmed.as_bytes().first()?;
    if mark != b'`' && mark != b'~' {
        return None;
    }
    let len = trimmed.bytes().take_while(|byte| *byte == mark).count();
    (len >= 3).then(|| (mark, len, &trimmed[len..]))
}

#[cfg(test)]
#[path = "code_fence_tests.rs"]
mod tests;
