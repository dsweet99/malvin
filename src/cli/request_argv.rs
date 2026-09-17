use std::ffi::OsString;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Do,
    Router,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedRequest {
    pub text: String,
    pub kind: RequestKind,
}

impl TaggedRequest {
    #[must_use]
    pub const fn is_do(&self) -> bool {
        matches!(self.kind, RequestKind::Do)
    }

    #[must_use]
    pub const fn is_router(&self) -> bool {
        matches!(self.kind, RequestKind::Router)
    }
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
    long_name(arg).is_some_and(|n| {
        matches!(
            n,
            "model" | "max-acp-retries" | "max-loops" | "max-hypotheses"
        )
    })
}

fn is_option_token(arg: &str) -> bool {
    arg == "-" || (arg.starts_with('-') && arg != "--")
}

fn skip_option(i: usize, arg: &str) -> usize {
    if opt_takes_following_value(arg) {
        i + 2
    } else {
        i + 1
    }
}

fn finish_pending_do(pending_do: bool, out: &[TaggedRequest]) -> Result<(), String> {
    if !pending_do || out.is_empty() {
        return Ok(());
    }
    Err("`--do` requires a REQUEST immediately after it".to_string())
}

fn push_request(out: &mut Vec<TaggedRequest>, text: &str, pending_do: &mut bool) {
    let kind = if *pending_do {
        *pending_do = false;
        RequestKind::Do
    } else {
        RequestKind::Router
    };
    out.push(TaggedRequest {
        text: text.to_string(),
        kind,
    });
}

pub fn classify_top_level_requests(args: &[OsString]) -> Result<Vec<TaggedRequest>, String> {
    let mut out = Vec::new();
    let mut pending_do = false;
    let mut i = 1usize;
    while i < args.len() {
        let arg = os_to_str(&args[i])?;
        if arg == "--do" {
            if pending_do {
                return Err("`--do` requires a REQUEST immediately after it".to_string());
            }
            pending_do = true;
            i += 1;
            continue;
        }
        if arg == "admin" {
            break;
        }
        if is_option_token(arg) {
            i = skip_option(i, arg);
            continue;
        }
        push_request(&mut out, arg, &mut pending_do);
        i += 1;
    }
    finish_pending_do(pending_do, &out)?;
    Ok(out)
}

#[cfg(test)]
#[path = "request_argv_tests.rs"]
mod tests;
