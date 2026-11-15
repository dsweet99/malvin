use std::ffi::OsString;

use malvin::config::malvin_config_file::parse_model_list_cli_arg;
use malvin::config::model_id::require_prefixed_model;
use malvin::config::model_list::ModelList;

pub(crate) struct ModelSegments {
    pub args: Vec<OsString>,
    in_effect: Vec<Option<String>>,
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

fn scan_model_flags(args: &[OsString]) -> Vec<ModelFlag> {
    let mut flags = Vec::new();
    let mut i = 1;
    while i < args.len() && args[i] != "--" {
        match model_flag_at(args, i) {
            Some(flag) => {
                i += flag.width;
                flags.push(flag);
            }
            None => i += 1,
        }
    }
    flags
}

pub(crate) fn split_repeated_model_flags(args: &[OsString]) -> Option<ModelSegments> {
    let flags = scan_model_flags(args);
    if flags.len() < 2 {
        return None;
    }
    let removed = |i: usize| {
        flags[1..]
            .iter()
            .any(|f| i >= f.start && i < f.start + f.width)
    };
    let mut segments = ModelSegments {
        args: Vec::new(),
        in_effect: Vec::new(),
    };
    for (i, arg) in args.iter().enumerate().filter(|(i, _)| !removed(*i)) {
        let current = flags.iter().rev().find(|f| f.start <= i);
        segments.args.push(arg.clone());
        segments.in_effect.push(current.map(|f| f.value.clone()));
    }
    Some(segments)
}

impl ModelSegments {
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
