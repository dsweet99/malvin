use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Router,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedRequest {
    pub text: String,
    pub kind: RequestKind,
}

impl TaggedRequest {
    #[must_use]
    pub const fn is_router(&self) -> bool {
        matches!(self.kind, RequestKind::Router)
    }
}

enum ArgStep {
    Break,
    Advance(usize),
    Request,
}

fn os_to_str(arg: &OsString) -> Result<&str, String> {
    arg.to_str()
        .ok_or_else(|| "malvin: command-line arguments must be valid UTF-8".to_string())
}

fn long_name(arg: &str) -> Option<&str> {
    arg.strip_prefix("--")
        .map(|rest| rest.split_once('=').map_or(rest, |(n, _)| n))
}

fn opt_takes_following_value(arg: &str) -> bool {
    if arg.contains('=') {
        return false;
    }
    long_name(arg).is_some_and(|n| matches!(n, "model" | "max-acp-retries" | "max-loops"))
}

fn advice_takes_following_value(arg: &str, next: Option<&str>) -> bool {
    if arg.contains('=') {
        return false;
    }
    if long_name(arg) != Some("advice") {
        return false;
    }
    next.is_some_and(|n| !is_option_token(n))
}

fn is_option_token(arg: &str) -> bool {
    arg == "-" || (arg.starts_with('-') && arg != "--")
}

fn skip_option(i: usize, arg: &str, args: &[OsString]) -> usize {
    let next = args.get(i + 1).and_then(|a| a.to_str());
    if opt_takes_following_value(arg) || advice_takes_following_value(arg, next) {
        i + 2
    } else {
        i + 1
    }
}

fn push_request(out: &mut Vec<TaggedRequest>, text: &str) {
    out.push(TaggedRequest {
        text: text.to_string(),
        kind: RequestKind::Router,
    });
}

fn classify_arg(arg: &str, i: usize, args: &[OsString]) -> ArgStep {
    if arg == "admin" {
        return ArgStep::Break;
    }
    if is_option_token(arg) {
        return ArgStep::Advance(skip_option(i, arg, args));
    }
    ArgStep::Request
}

pub fn classify_top_level_requests(args: &[OsString]) -> Result<Vec<TaggedRequest>, String> {
    let mut out = Vec::new();
    let mut positional_only = false;
    let mut i = 1usize;
    while i < args.len() {
        let arg = os_to_str(&args[i])?;
        if !positional_only && arg == "--" {
            positional_only = true;
            i += 1;
            continue;
        }
        if positional_only {
            push_request(&mut out, arg);
            i += 1;
            continue;
        }
        match classify_arg(arg, i, args) {
            ArgStep::Break => break,
            ArgStep::Advance(next) => i = next,
            ArgStep::Request => {
                push_request(&mut out, arg);
                i += 1;
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
#[path = "request_argv_tests.rs"]
mod tests;
