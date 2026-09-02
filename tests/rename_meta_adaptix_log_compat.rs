mod common;

use std::path::Path;

use clap::Parser;
use malvin::cli::Cli;
use malvin::output::{MALVIN_WHO, WHO_U, format_line, format_who_tag_prefix};
use malvin::{MALVIN_USER_HOME_DIR, workspace_logs_hash};

const LEGACY_RUN: &str = "20250101_120000_legacy01";
const LEGACY_ID: &str = "Mleg01";

fn legacy_run_dir(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    home.join(MALVIN_USER_HOME_DIR)
        .join("logs")
        .join(workspace_logs_hash(work_dir))
        .join(LEGACY_RUN)
}

fn write_legacy_run_transcripts(run_dir: &Path) {
    let run_meta_dir = run_dir.join("_run");
    std::fs::create_dir_all(&run_meta_dir).expect("mkdir run meta");
    let exp_name = format!("exp_log_{LEGACY_RUN}.md");
    std::fs::write(run_meta_dir.join(&exp_name), "legacy exp log\n").expect("write exp");
    let exp_rel = format!("./{}/_run/{exp_name}", run_dir.display());
    std::fs::write(
        run_dir.join("stdout.log"),
        format!(
            "20250101.120000.000 {}EXP_LOG: {LEGACY_ID} {exp_rel}\n",
            format_who_tag_prefix(MALVIN_WHO)
        ),
    )
    .expect("stdout");
    std::fs::write(
        run_dir.join("command.log"),
        format!(
            "{}\n",
            format_line(WHO_U, "Command: malvin adaptix legacy ideation prompt")
        ),
    )
    .expect("command.log");
    std::fs::write(run_dir.join("adaptix.log"), "legacy inspire transcript\n")
        .expect("adaptix.log");
}

fn write_legacy_workspace_meta_toml(work_dir: &Path) {
    let malvin_dir = work_dir.join(".malvin");
    std::fs::create_dir_all(&malvin_dir).expect("mkdir .malvin");
    std::fs::write(malvin_dir.join("meta.toml"), "theme = \"dark\"\n").expect("meta.toml");
}

fn seed_legacy_adaptix_run(work_dir: &Path, home: &Path) -> std::path::PathBuf {
    let run_dir = legacy_run_dir(work_dir, home);
    write_legacy_run_transcripts(&run_dir);
    malvin::write_work_dir_manifest(&run_dir, work_dir).expect("manifest");
    write_legacy_workspace_meta_toml(work_dir);
    run_dir
}

#[test]
fn adaptix_subcommand_is_removed() {
    use clap::CommandFactory;
    assert!(
        !Cli::command()
            .get_subcommands()
            .any(|c| c.get_name() == "adaptix" || c.get_name() == "inspire"),
        "inspire/adaptix must not be clap subcommands"
    );
    let cli = Cli::try_parse_from(["malvin", "adaptix"]).expect("bare request");
    assert!(cli.command.is_none());
    assert_eq!(cli.request.as_deref(), Some("adaptix"));
}

#[test]
fn legacy_adaptix_run_dir_is_listed_in_home_bucket() {
    common::with_isolated_home(|work, home| {
        let run_dir = seed_legacy_adaptix_run(work, home);
        assert!(
            run_dir.is_dir(),
            "legacy run dir must exist at {}",
            run_dir.display()
        );
        let names: Vec<_> = std::fs::read_dir(run_dir.parent().unwrap())
            .expect("read bucket")
            .map(|e| e.expect("entry").file_name())
            .collect();
        assert!(
            names.iter().any(|n| n.to_string_lossy() == LEGACY_RUN),
            "legacy run must appear in bucket listing: {names:?}"
        );
    });
}
