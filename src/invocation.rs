//! Process invocation line for inclusion in ACP trace logs and `command.log`.

use std::sync::OnceLock;

static COMMAND_LINE: OnceLock<String> = OnceLock::new();

/// Records [`std::env::args`] joined with ASCII spaces, once per process.
///
/// This is a conventional display string for logs and UI, **not** a lossless or shell-pastable
/// reconstruction: arguments that contain spaces, quotes, or other shell metacharacters cannot be
/// round-tripped into an equivalent shell invocation from this string alone.
pub fn init_from_env() {
    let _ = COMMAND_LINE.get_or_init(|| std::env::args().collect::<Vec<_>>().join(" "));
}

/// Full command line captured by [`init_from_env`], if initialized.
#[must_use]
pub fn command_line() -> Option<&'static str> {
    COMMAND_LINE.get().map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_records_argv() {
        init_from_env();
        let line = command_line().expect("command line after init");
        assert!(!line.is_empty());
    }
}
