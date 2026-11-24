use super::{fact_trace_path, worker_args, worker_command};
use crate::artifacts::RunArtifacts;
use std::ffi::OsString;

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn worker_args_parse_and_command_round_trip() {
    assert!(worker_args(os(&[])).is_err());
    let missing = worker_args(os(&["/r/p.md"]));
    assert!(matches!(missing, Err(e) if e.contains("usage") && e.contains("__fact-agent-worker")));

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
    let args = command.as_std().get_args().map(OsString::from);
    let (worker, rest) = crate::self_worker::invocation(args).unwrap();
    assert_eq!(worker, crate::self_worker::Worker::FactAgent);
    assert_eq!(worker_args(rest), Ok((prompt, models.cli_value())));
    let trace = fact_trace_path(&artifacts).unwrap();
    assert_eq!(trace, dir.path().join("fact/trace.jsonl"));
    assert!(dir.path().join("fact").is_dir());
    let strip = crate::self_exe::strip_deleted_suffix;
    assert_eq!(
        strip("/a/malvin (deleted)".into()),
        std::path::PathBuf::from("/a/malvin")
    );
    assert_eq!(
        strip("/a/malvin".into()),
        std::path::PathBuf::from("/a/malvin")
    );
    assert!(crate::self_exe::self_exe().unwrap().exists());
}
