use std::collections::BTreeMap;

use clap::Args;
use malvin::modal_run::gpu_types::{GPU_TYPES_URL, GpuTypes, load_gpu_types};
use malvin::modal_run::options::{SUBOPTIONS, Suboption};
use malvin::modal_run::{MODAL_ABOUT, MODAL_REMOTE_ID, REMOTE_FLAG};
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Args, Debug, Clone, Default)]
#[command(override_usage = "malvin admin remotes [OPTION]...")]
pub struct RemotesArgs {
    /// Force-refetch Modal's GPU types (also runs automatically every 24h).
    #[arg(long)]
    pub refresh: bool,
}

fn summary_row() -> String {
    let keys: Vec<String> = SUBOPTIONS
        .iter()
        .map(|s| format!("{}={}", s.key, s.values))
        .collect();
    format!("{MODAL_REMOTE_ID}\t{}", keys.join(" "))
}

fn column_width(header: &str, cell: fn(&Suboption) -> &'static str) -> usize {
    SUBOPTIONS
        .iter()
        .map(|s| cell(s).len())
        .fold(header.len(), usize::max)
}

fn suboption_table() -> Vec<String> {
    let key_w = column_width("KEY", |s| s.key);
    let val_w = column_width("VALUES", |s| s.values);
    let def_w = column_width("DEFAULT", |s| s.default);
    let row = |k: &str, v: &str, d: &str, m: &str| {
        format!("  {k:<key_w$}  {v:<val_w$}  {d:<def_w$}  {m}")
    };
    let mut lines = vec![row("KEY", "VALUES", "DEFAULT", "MEANING")];
    for s in &SUBOPTIONS {
        let (first, rest) = s
            .meaning
            .split_first()
            .map_or(("", &[][..]), |(f, r)| (*f, r));
        lines.push(row(s.key, s.values, s.default, first));
        lines.extend(rest.iter().map(|m| row("", "", "", m)));
    }
    lines
}

fn gpu_type_lines(gpu: &GpuTypes) -> Vec<String> {
    let mut lines = Vec::new();
    if !gpu.types.is_empty() {
        lines.push(format!("GPU types: {}", gpu.types.join(", ")));
    }
    lines.extend(gpu.note.iter().map(|note| format!("GPU types: {note}")));
    lines.push(format!(
        "(from {GPU_TYPES_URL}; refreshed daily or with --refresh)"
    ));
    lines
}

#[must_use]
pub fn remotes_lines(gpu: &GpuTypes) -> Vec<String> {
    let mut lines = vec![
        summary_row(),
        String::new(),
        format!("{MODAL_REMOTE_ID}: {MODAL_ABOUT}."),
        format!(
            "Usage: malvin {REMOTE_FLAG}={MODAL_REMOTE_ID}[KEY=VALUE,...] [OPTION]... [REQUEST]..."
        ),
        String::new(),
        "Suboptions (each optional, in any order; an omitted one falls back to the same"
            .to_string(),
        "key under [modal] in ~/.malvinconf/config.toml, then to DEFAULT):".to_string(),
    ];
    lines.extend(suboption_table());
    lines.push(String::new());
    lines.extend(gpu_type_lines(gpu));
    lines.extend([
        String::new(),
        format!("Example: malvin '{REMOTE_FLAG}={MODAL_REMOTE_ID}[gpu=A100,mem=32,timeout=2h]' \"Train the model\""),
        "Quote the flag: brackets are shell glob characters.".to_string(),
    ]);
    lines
}

pub(crate) fn remote_listing(gpu: &GpuTypes, aliases: &BTreeMap<String, String>) -> Vec<String> {
    let mut lines = remotes_lines(gpu);
    lines.extend(super::config_alias_list::alias_section_lines(
        super::config_alias_list::REMOTE_ALIAS_SECTION,
        aliases,
    ));
    lines
}

pub fn run_remotes(args: &RemotesArgs) {
    for line in remote_listing(
        &load_gpu_types(args.refresh),
        &malvin::config::malvin_config_file::load_remote_aliases(),
    ) {
        print_stdout_line(MALVIN_WHO, &line);
    }
}

#[cfg(test)]
mod tests {
    use super::{GpuTypes, remote_listing, remotes_lines};

    fn listed() -> GpuTypes {
        GpuTypes {
            types: vec!["T4".to_string(), "H100".to_string()],
            note: None,
        }
    }
    #[test]
    fn remotes_shows_fetched_gpu_types_and_their_source_and_remotes_shows_the_note_when_gpu_types_are_unavailable()
     {
        {
            let text = remotes_lines(&listed()).join("\n");
            assert!(
                text.contains("GPU types: T4, H100\n(from https://modal.com/docs/guide/gpu.md"),
                "{text}"
            );
        }
        {
            let gpu = GpuTypes {
                types: Vec::new(),
                note: Some("could not fetch".to_string()),
            };
            let text = remotes_lines(&gpu).join("\n");
            assert!(text.contains("GPU types: could not fetch"), "{text}");
            assert!(!text.contains("GPU types: \n"), "{text}");
        }
        remotes_summary_matches_the_admin_doc_example();
    }

    fn remotes_summary_matches_the_admin_doc_example() {
        let doc = include_str!("../../default_prompts/docs/admin.md");
        let example = doc
            .split("malvin admin models` (")
            .nth(1)
            .and_then(|rest| rest.split("`)").next())
            .expect("admin doc example");
        let expected = example.trim_matches('`').replace("<TAB>", "\t");
        let actual = remotes_lines(&listed())
            .into_iter()
            .next()
            .expect("summary");
        assert_eq!(actual, expected, "doc example {example:?}");
    }

    #[test]
    fn remotes_lists_modal_and_every_suboption() {
        let text = remotes_lines(&listed()).join("\n");
        assert!(
            text.starts_with(
                "modal:sandbox\tgpu=none|TYPE[:COUNT] ncpu=N mem=N[G|GB|GiB] timeout=N[s|m|h]"
            ),
            "{text}"
        );
        for needle in [
            "  gpu ",
            "  ncpu ",
            "  mem ",
            "  timeout ",
            "H100",
            "--remote=modal:sandbox[KEY=VALUE,...]",
            "30m",
        ] {
            assert!(text.contains(needle), "missing {needle}: {text}");
        }
    }

    #[test]
    fn remotes_listing_appends_config_aliases_after_the_example() {
        use std::collections::BTreeMap;
        let gpu = listed();
        assert_eq!(remote_listing(&gpu, &BTreeMap::new()), remotes_lines(&gpu));
        let aliases = BTreeMap::from([
            ("big".to_string(), "modal:sandbox[gpu=A100]".to_string()),
            ("small".to_string(), "modal:sandbox[ncpu=1]".to_string()),
        ]);
        let lines = remote_listing(&gpu, &aliases);
        let example = lines
            .iter()
            .position(|line| line.starts_with("Example:"))
            .expect("example");
        let header = lines
            .iter()
            .position(|line| line.contains("[aliases.remotes]"))
            .expect("alias header");
        assert!(header > example, "{lines:?}");
        assert_eq!(lines[header + 1], "big\tmodal:sandbox[gpu=A100]");
        assert_eq!(lines[header + 2], "small\tmodal:sandbox[ncpu=1]");
        assert_eq!(lines.len(), header + 3);
    }
}
