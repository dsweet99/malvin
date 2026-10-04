use malvin::modal_run::options::{GPU_TYPES, SUBOPTIONS, Suboption};
use malvin::modal_run::{MODAL_ABOUT, MODAL_REMOTE, REMOTE_FLAG};
use malvin::output::{MALVIN_WHO, print_stdout_line};

fn summary_row() -> String {
    let keys: Vec<String> = SUBOPTIONS
        .iter()
        .map(|s| format!("{}={}", s.key, s.values))
        .collect();
    format!("{MODAL_REMOTE}\t{}", keys.join(" "))
}

fn column_width(header: &str, cell: fn(&Suboption) -> &'static str) -> usize {
    SUBOPTIONS.iter().map(|s| cell(s).len()).fold(header.len(), usize::max)
}

fn suboption_table() -> Vec<String> {
    let key_w = column_width("KEY", |s| s.key);
    let val_w = column_width("VALUES", |s| s.values);
    let def_w = column_width("DEFAULT", |s| s.default);
    let row = |k: &str, v: &str, d: &str, m: &str| format!("  {k:<key_w$}  {v:<val_w$}  {d:<def_w$}  {m}");
    let mut lines = vec![row("KEY", "VALUES", "DEFAULT", "MEANING")];
    for s in &SUBOPTIONS {
        let (first, rest) = s.meaning.split_first().map_or(("", &[][..]), |(f, r)| (*f, r));
        lines.push(row(s.key, s.values, s.default, first));
        lines.extend(rest.iter().map(|m| row("", "", "", m)));
    }
    lines
}

#[must_use]
pub fn remotes_lines() -> Vec<String> {
    let mut lines = vec![
        summary_row(),
        String::new(),
        format!("{MODAL_REMOTE}: {MODAL_ABOUT}."),
        format!("Usage: malvin {REMOTE_FLAG}={MODAL_REMOTE}[KEY=VALUE,...] [OPTION]... [REQUEST]..."),
        String::new(),
        "Suboptions (each optional, in any order; an omitted one falls back to the same".to_string(),
        "key under [modal] in ~/.malvinconf/config.toml, then to DEFAULT):".to_string(),
    ];
    lines.extend(suboption_table());
    lines.extend([
        String::new(),
        format!("GPU types: {GPU_TYPES}"),
        String::new(),
        format!("Example: malvin '{REMOTE_FLAG}={MODAL_REMOTE}[gpu=A100,mem=32,timeout=2h]' \"Train the model\""),
        "Quote the flag: brackets are shell glob characters.".to_string(),
    ]);
    lines
}

pub fn run_remotes() {
    for line in remotes_lines() {
        print_stdout_line(MALVIN_WHO, &line);
    }
}

#[cfg(test)]
mod tests {
    use super::remotes_lines;

    #[test]
    fn remotes_lists_modal_and_every_suboption() {
        let text = remotes_lines().join("\n");
        assert!(text.starts_with("modal\tgpu=none|TYPE[:COUNT] ncpu=N mem=N[G|GB|GiB] timeout=N[s|m|h]"), "{text}");
        for needle in ["  gpu ", "  ncpu ", "  mem ", "  timeout ", "H100", "--remote=modal[KEY=VALUE,...]", "30m"] {
            assert!(text.contains(needle), "missing {needle}: {text}");
        }
    }
}
