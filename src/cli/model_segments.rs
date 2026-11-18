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

fn gates_flag_at(args: &[OsString], i: usize) -> Option<ModelFlag> {
    let text = args[i].to_str()?;
    matches!(text, "-g" | "--gates").then(|| ModelFlag {
        start: i,
        width: 1,
        value: String::new(),
    })
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
    for i in (0..args.len()).filter(|&i| !is_removed(&flags, i) && !is_removed(&gates, i)) {
        let model = model_in_effect(&flags, i);
        let on = gates.first().is_some_and(|g| g.start <= i);
        segments.push(&args[i], model, on);
    }
    Some(segments)
}

impl ModelSegments {
    fn push(&mut self, arg: &OsString, model: Option<String>, on: bool) {
        self.args.push(arg.clone());
        self.in_effect.push(model);
        self.gates_in_effect.push(on);
    }

    pub(crate) fn arg_positions(&self, clap_indices: &[usize]) -> Vec<usize> {
        let mut starts = Vec::with_capacity(self.args.len());
        let mut next = 0;
        for arg in &self.args {
            starts.push(next);
            let split = arg
                .to_str()
                .is_some_and(|t| t.starts_with("--") && t.contains('='));
            next += if split { 2 } else { 1 };
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
