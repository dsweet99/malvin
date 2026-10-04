use std::io::Write;

use crate::output::{ERROR_WHO, MALVIN_WHO, WHO_R, print_stderr_line, print_stdout_line};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Stream {
    Stdout,
    Stderr,
}

#[derive(Default)]
pub(super) struct RemoteLines {
    stdout: String,
    stderr: String,
}

impl RemoteLines {
    const fn buf(&mut self, stream: Stream) -> &mut String {
        match stream {
            Stream::Stdout => &mut self.stdout,
            Stream::Stderr => &mut self.stderr,
        }
    }

    pub(super) fn push(&mut self, stream: Stream, data: &str) {
        let buf = self.buf(stream);
        buf.push_str(data);
        let Some(end) = buf.rfind('\n') else {
            return;
        };
        let complete: String = buf.drain(..=end).collect();
        for line in complete.lines() {
            emit(stream, line);
        }
    }

    pub(super) fn flush(&mut self) {
        for stream in [Stream::Stdout, Stream::Stderr] {
            let rest = std::mem::take(self.buf(stream));
            let rest = rest.strip_suffix('\r').unwrap_or(&rest);
            if !rest.is_empty() {
                emit(stream, rest);
            }
        }
    }
}

pub(super) fn is_tagged(line: &str) -> bool {
    let plain = crate::ansi_strip::strip_ansi_escapes(line);
    let mut chars = plain.chars();
    matches!((chars.next(), chars.next()), (Some(c), Some('|')) if c.is_ascii_lowercase())
}

pub(super) fn print_status(line: &str) {
    if !crate::output::do_dm_stdout_mode() {
        print_stderr_line(MALVIN_WHO, line);
    }
}

pub(super) fn is_error_line(line: &str) -> bool {
    crate::ansi_strip::strip_ansi_escapes(line)
        .strip_prefix(ERROR_WHO)
        .is_some_and(|rest| rest.starts_with('|'))
}

fn emit_do_dm(stream: Stream, line: &str) {
    if is_error_line(line) {
        write_raw(&mut std::io::stderr().lock(), line);
    } else if stream == Stream::Stdout && !is_tagged(line) {
        crate::output::emit_wrapped_do_dm_line(line);
    }
}

fn emit(stream: Stream, line: &str) {
    if crate::output::do_dm_stdout_mode() {
        emit_do_dm(stream, line);
        return;
    }
    match (stream, is_tagged(line)) {
        (Stream::Stdout, false) => print_stdout_line(WHO_R, line),
        (Stream::Stderr, false) => print_stderr_line(WHO_R, line),
        (Stream::Stdout, true) => write_raw(&mut std::io::stdout().lock(), line),
        (Stream::Stderr, true) => write_raw(&mut std::io::stderr().lock(), line),
    }
}

fn write_raw(out: &mut impl Write, line: &str) {
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}
