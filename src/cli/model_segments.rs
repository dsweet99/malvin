use std::ffi::OsString;

use malvin::config::malvin_config_file::parse_model_list_cli_arg;
use malvin::config::model_id::require_prefixed_model;
use malvin::config::model_list::ModelList;

pub(crate) struct ModelSegments {
    pub args: Vec<OsString>,
    in_effect: Vec<Option<String>>,
    gates_in_effect: Vec<bool>,
    multi_model: bool,
}

struct ModelFlag {
    start: usize,
    width: usize,
    value: String,
}

fn model_flag_at(args: &[OsString], i: usize) -> Option<ModelFlag> {
    let text = args[i].to_str()?;
    if let Some(v) = text.strip_prefix("--model=") {
        return Some(ModelFlag {
            start: i,
            width: 1,
            value: v.to_string(),
        });
    }
    let next = args.get(i + 1)?.to_str()?;
    (text == "--model").then(|| ModelFlag {
        start: i,
        width: 2,
        value: next.to_string(),
    })
}

fn scan_flags(
    args: &[OsString],
    at: fn(&[OsString], usize) -> Option<ModelFlag>,
) -> Vec<ModelFlag> {
    let mut flags = Vec::new();
    let mut i = 1;
    while i < args.len() && args[i] != "--" {
        match at(args, i) {
            Some(flag) => {
                i += flag.width;
                flags.push(flag);
            }
            None => i += 1,
        }
    }
    flags
}

fn is_short_cluster(text: &str) -> bool {
    text.len() > 1 && text.starts_with('-') && !text.starts_with("--")
}

fn gates_flag_at(args: &[OsString], i: usize) -> Option<ModelFlag> {
    let text = args[i].to_str()?;
    let bundled = is_short_cluster(text) && text[1..].contains('g');
    (text == "--gates" || bundled).then(|| ModelFlag {
        start: i,
        width: 1,
        value: String::new(),
    })
}

fn without_gates(arg: &OsString) -> Option<OsString> {
    let text = arg.to_str()?;
    if text == "--gates" {
        return None;
    }
    let rest: String = text[1..].chars().filter(|&c| c != 'g').collect();
    (!rest.is_empty()).then(|| OsString::from(format!("-{rest}")))
}

fn clap_width(text: &str, after_double_dash: bool) -> usize {
    if after_double_dash {
        1
    } else if text == "--" {
        0
    } else if text.starts_with("--") {
        if text.contains('=') { 2 } else { 1 }
    } else if is_short_cluster(text) {
        text.chars().count() - 1
    } else {
        1
    }
}
fn model_in_effect(flags: &[ModelFlag], i: usize) -> Option<String> {
    flags
        .iter()
        .rev()
        .find(|f| f.start <= i)
        .map(|f| f.value.clone())
}

fn is_removed(list: &[ModelFlag], i: usize) -> bool {
    list.iter()
        .skip(1)
        .any(|f| i >= f.start && i < f.start + f.width)
}

pub(crate) fn split_repeated_model_flags(args: &[OsString]) -> Option<ModelSegments> {
    let flags = scan_flags(args, model_flag_at);
    let gates = scan_flags(args, gates_flag_at);
    if flags.len() < 2 && gates.is_empty() {
        return None;
    }
    let mut segments = ModelSegments {
        args: Vec::new(),
        in_effect: Vec::new(),
        gates_in_effect: Vec::new(),
        multi_model: flags.len() >= 2,
    };
    for i in (0..args.len()).filter(|&i| !is_removed(&flags, i)) {
        let arg = if is_removed(&gates, i) {
            without_gates(&args[i])
        } else {
            Some(args[i].clone())
        };
        let Some(arg) = arg else { continue };
        let model = model_in_effect(&flags, i);
        let on = gates.first().is_some_and(|g| g.start <= i);
        segments.push(arg, model, on);
    }
    Some(segments)
}

impl ModelSegments {
    fn push(&mut self, arg: OsString, model: Option<String>, on: bool) {
        self.args.push(arg);
        self.in_effect.push(model);
        self.gates_in_effect.push(on);
    }

    pub(crate) fn arg_positions(&self, clap_indices: &[usize]) -> Vec<usize> {
        let mut starts = Vec::with_capacity(self.args.len());
        let mut next = 0;
        let mut after_double_dash = false;
        for arg in &self.args {
            let text = arg.to_str().unwrap_or("");
            let width = clap_width(text, after_double_dash);
            starts.push(if width == 0 { usize::MAX } else { next });
            next += width;
            after_double_dash |= text == "--";
        }
        clap_indices
            .iter()
            .filter_map(|c| starts.iter().position(|s| s == c))
            .collect()
    }
    pub(crate) fn gates_at(&self, indices: &[usize]) -> Vec<bool> {
        indices
            .iter()
            .map(|&i| self.gates_in_effect.get(i).copied().unwrap_or(false))
            .collect()
    }

    pub(crate) const fn has_multiple_models(&self) -> bool {
        self.multi_model
    }

    pub(crate) fn models_at(
        &self,
        indices: &[usize],
        first: &ModelList,
    ) -> Result<Vec<ModelList>, String> {
        indices
            .iter()
            .map(
                |&idx| match self.in_effect.get(idx).and_then(Option::as_deref) {
                    None => Ok(first.clone()),
                    Some(text) => {
                        let list = parse_model_list_cli_arg(text)?;
                        require_prefixed_model(&list.first().canonical())?;
                        Ok(list)
                    }
                },
            )
            .collect()
    }
}
