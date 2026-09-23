use std::ffi::OsString;

use super::shared_opts::parse_creative_probability;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Do,
    Router,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TaggedRequest {
    pub text: String,
    pub kind: RequestKind,
    pub creative: Option<f64>,
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

#[derive(Debug, Clone, Copy, Default)]
struct PendingPrefixes {
    do_workflow: bool,
    creative: Option<f64>,
}

impl PendingPrefixes {
    fn set_do(&mut self) -> Result<(), String> {
        if self.do_workflow || self.creative.is_some() {
            return Err(self.conflict_message());
        }
        self.do_workflow = true;
        Ok(())
    }

    fn set_creative(&mut self, prob: f64) -> Result<(), String> {
        if self.do_workflow || self.creative.is_some() {
            return Err(self.conflict_message());
        }
        self.creative = Some(prob);
        Ok(())
    }

    fn conflict_message(&self) -> String {
        if self.do_workflow {
            "`--do` requires a REQUEST immediately after it".to_string()
        } else {
            "`--creative` requires a REQUEST immediately after it".to_string()
        }
    }

    fn finish(self, out: &[TaggedRequest]) -> Result<(), String> {
        if out.is_empty() {
            return Ok(());
        }
        if self.do_workflow || self.creative.is_some() {
            return Err(self.conflict_message());
        }
        Ok(())
    }

    const fn take_for_request(&mut self) -> (RequestKind, Option<f64>) {
        let kind = if self.do_workflow {
            self.do_workflow = false;
            RequestKind::Do
        } else {
            RequestKind::Router
        };
        let creative = self.creative.take();
        (kind, creative)
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

fn parse_creative_flag(arg: &str) -> Result<Option<f64>, String> {
    if arg == "--creative" {
        return Ok(Some(1.0));
    }
    if let Some(rest) = arg.strip_prefix("--creative=") {
        return parse_creative_probability(rest).map(Some);
    }
    Ok(None)
}

fn push_request(out: &mut Vec<TaggedRequest>, text: &str, pending: &mut PendingPrefixes) {
    let (kind, creative) = pending.take_for_request();
    out.push(TaggedRequest {
        text: text.to_string(),
        kind,
        creative,
    });
}

fn classify_arg(arg: &str, i: usize, pending: &mut PendingPrefixes) -> Result<ArgStep, String> {
    if let Some(p) = parse_creative_flag(arg)? {
        pending.set_creative(p)?;
        return Ok(ArgStep::Advance(i + 1));
    }
    if arg == "--do" {
        pending.set_do()?;
        return Ok(ArgStep::Advance(i + 1));
    }
    if arg == "admin" {
        return Ok(ArgStep::Break);
    }
    if is_option_token(arg) {
        return Ok(ArgStep::Advance(skip_option(i, arg)));
    }
    Ok(ArgStep::Request)
}

pub fn classify_top_level_requests(args: &[OsString]) -> Result<Vec<TaggedRequest>, String> {
    let mut out = Vec::new();
    let mut pending = PendingPrefixes::default();
    let mut i = 1usize;
    while i < args.len() {
        let arg = os_to_str(&args[i])?;
        match classify_arg(arg, i, &mut pending)? {
            ArgStep::Break => break,
            ArgStep::Advance(next) => i = next,
            ArgStep::Request => {
                push_request(&mut out, arg, &mut pending);
                i += 1;
            }
        }
    }
    pending.finish(&out)?;
    Ok(out)
}

#[cfg(test)]
#[path = "request_argv_tests.rs"]
mod tests;
