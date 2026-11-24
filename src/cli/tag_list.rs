use std::io::{self, Write};

use crate::desc_limit::MAX_DESC_WORDS;

fn out_err(e: &io::Error) -> String {
    format!("stdout: {e}")
}

pub(crate) fn limit_words(text: &str) -> String {
    text.split_whitespace()
        .take(MAX_DESC_WORDS)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn list_line(tag: &str, description: &str) -> String {
    format!("{tag}: {description}")
}

pub(crate) fn write_list(mut out: impl Write, flag: &str, lines: &[String]) -> Result<(), String> {
    write!(
        out,
        "Type `malvin {flag} TAG` to see a full document.\n\nTAG: Description\n"
    )
    .map_err(|e| out_err(&e))?;
    for line in lines {
        writeln!(out, "{line}").map_err(|e| out_err(&e))?;
    }
    writeln!(out).map_err(|e| out_err(&e))
}

pub(crate) fn write_body(mut out: impl Write, text: &str) -> Result<(), String> {
    out.write_all(text.as_bytes()).map_err(|e| out_err(&e))?;
    if !text.ends_with('\n') {
        out.write_all(b"\n").map_err(|e| out_err(&e))?;
    }
    Ok(())
}
