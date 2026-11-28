use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StdoutMode {
    #[default]
    Normal,
    DmOnly {
        markdown: bool,
    },
    Silent,
}

impl StdoutMode {
    const fn code(self) -> u8 {
        match self {
            Self::Normal => 0,
            Self::DmOnly { markdown: false } => 1,
            Self::DmOnly { markdown: true } => 2,
            Self::Silent => 3,
        }
    }

    const fn from_code(code: u8) -> Self {
        match code {
            1 => Self::DmOnly { markdown: false },
            2 => Self::DmOnly { markdown: true },
            3 => Self::Silent,
            _ => Self::Normal,
        }
    }

    const fn is_dm_only(self) -> bool {
        matches!(self, Self::DmOnly { .. })
    }
}

static MODE: AtomicU8 = AtomicU8::new(0);

#[cfg(test)]
pub(crate) static DO_DM_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[must_use]
pub fn stdout_mode() -> StdoutMode {
    StdoutMode::from_code(MODE.load(Ordering::Relaxed))
}

pub fn set_stdout_mode(mode: StdoutMode) {
    MODE.store(mode.code(), Ordering::Relaxed);
}

#[must_use = "the previous stdout mode is restored when the guard is dropped"]
pub struct StdoutModeGuard {
    previous: StdoutMode,
}

impl Drop for StdoutModeGuard {
    fn drop(&mut self) {
        MODE.store(self.previous.code(), Ordering::Relaxed);
    }
}

pub fn enter_stdout_mode(mode: StdoutMode) -> StdoutModeGuard {
    let previous = stdout_mode();
    set_stdout_mode(mode);
    StdoutModeGuard { previous }
}

#[must_use]
pub fn do_dm_stdout_mode() -> bool {
    stdout_mode().is_dm_only()
}

pub(crate) fn do_dm_stdout_markdown() -> bool {
    stdout_mode() == StdoutMode::DmOnly { markdown: true }
}

#[must_use]
pub fn stdout_display_discarded() -> bool {
    stdout_mode() == StdoutMode::Silent
}

#[must_use]
pub fn heartbeat_stdout_suppressed() -> bool {
    stdout_mode() != StdoutMode::Normal
}

#[cfg(test)]
#[path = "stdout_mode_tests.rs"]
mod tests;
