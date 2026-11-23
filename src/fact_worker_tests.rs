use super::{WORKER_ARG, fact_trace_path, strip_deleted_suffix, worker_args, worker_command};
use crate::artifacts::RunArtifacts;
use std::ffi::OsString;
use std::path::PathBuf;

fn os(args: &[&str]) -> impl Iterator<Item = OsString> {
    args.iter()
        .map(OsString::from)
        .collect::<Vec<_>>()
        .into_iter()
}

#[test]
fn worker_args_parse_and_command_round_trip() {
    assert!(worker_args(os(&[])).is_none());
    assert!(worker_args(os(&["--model=opus", "hi"])).is_none());
    let missing = worker_args(os(&[WORKER_ARG, "/r/p.md"]));
    assert!(matches!(missing, Some(Err(e)) if e.contains("usage")));

    let dir = tempfile::tempdir().unwrap();
    let artifacts = RunArtifacts {
        run_dir: dir.path().to_path_buf(),
        plan_path: dir.path().join("plan.md"),
        work_dir: dir.path().to_path_buf(),
    };
    let models = crate::config::malvin_config_file::parse_model_list_cli_arg(
        crate::config::DEFAULT_CLI_MODEL,
    )
    .unwrap();
    let prompt = dir.path().join("p.md");
    let command = worker_command(&models, &artifacts, &prompt, &dir.path().join("w.log")).unwrap();
    assert_eq!(command.as_std().get_current_dir(), Some(dir.path()));
    let parsed = worker_args(command.as_std().get_args().map(OsString::from));
    assert_eq!(parsed, Some(Ok((prompt, models.cli_value()))));
    let trace = fact_trace_path(&artifacts).unwrap();
    assert_eq!(trace, dir.path().join("fact/trace.jsonl"));
    assert!(dir.path().join("fact").is_dir());
    assert_eq!(
        strip_deleted_suffix(PathBuf::from("/a/malvin (deleted)")),
        PathBuf::from("/a/malvin")
    );
    assert_eq!(
        strip_deleted_suffix(PathBuf::from("/a/malvin")),
        PathBuf::from("/a/malvin")
    );
}
