use std::io::Write;
use std::path::{Path, PathBuf};

use super::bug_id_lookup::lookup_run_by_log_kind;
use super::bug_id_lookup::MalvinRunLogKind;

pub(crate) fn lookup_kpop_id(cwd: &Path, id: &str) -> Result<PathBuf, String> {
    crate::validate_malvin_short_id(id)?;
    let resolved = lookup_run_by_log_kind(cwd, id, MalvinRunLogKind::Kpop)?;
    Ok(resolved.exp_log_path)
}

pub(crate) fn dump_kpop_log_to_stdout(exp_log_path: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(exp_log_path).map_err(|e| e.to_string())?;
    let mut out = std::io::stdout().lock();
    out.write_all(text.as_bytes()).map_err(|e| e.to_string())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n").map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn kpop_log_line(
    id: &str,
    work_dir: &Path,
    run_dir: &Path,
    exp_log_path: &Path,
) -> String {
    let rel = crate::orchestrator::format_exp_log_relative(
        &crate::artifacts::RunArtifacts {
            run_dir: run_dir.to_path_buf(),
            plan_path: run_dir.join("plan.md"),
            work_dir: work_dir.to_path_buf(),
        },
        exp_log_path,
    );
    format!("KPOP_LOG: {id} {rel}")
}

pub(crate) fn is_kpop_lookup_request(request: Option<&str>) -> bool {
    request.is_some_and(|s| {
        let t = s.trim();
        !t.is_empty() && crate::is_valid_malvin_short_id(t)
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        dump_kpop_log_to_stdout, is_kpop_lookup_request, kpop_log_line, lookup_kpop_id,
    };
    use crate::output::{format_who_tag_prefix, MALVIN_WHO};

    fn seed_home_run(
        home: &Path,
        cwd: &Path,
        run_name: &str,
    ) -> (PathBuf, PathBuf) {
        let bucket = home.join(".malvin/logs").join(crate::workspace_logs_hash(cwd));
        let run_dir = bucket.join(run_name);
        std::fs::create_dir_all(&run_dir).expect("mkdir run");
        crate::write_work_dir_manifest(&run_dir, cwd).expect("work_dir manifest");
        let exp = run_dir.join("_kpop").join(format!("exp_log_{run_name}.md"));
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir kpop");
        (run_dir, exp)
    }

    fn with_test_home<F: FnOnce(&Path, &Path)>(f: F) {
        crate::test_utils::with_isolated_home(|cwd| {
            f(&crate::user_home_dir(), cwd);
        });
    }

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = lookup_kpop_id;
        let _ = dump_kpop_log_to_stdout;
        let _ = kpop_log_line;
        let _ = is_kpop_lookup_request;
    }

    #[test]
    fn kpop_lookup_finds_unique_kpop_log_line() {
        with_test_home(|home, cwd| {
            let (run_dir, exp) = seed_home_run(home, cwd, "20260101_000000_abcabcab");
            std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write exp");
            let rel = format!(
                "{}/_kpop/exp_log_20260101_000000_abcabcab.md",
                run_dir.display()
            );
            std::fs::write(
                run_dir.join("stdout.log"),
                format!(
                    "20260101.000000.000 {}KPOP_LOG: Ma1b2c {rel}\n",
                    format_who_tag_prefix(MALVIN_WHO)
                ),
            )
            .expect("stdout");
            let path = lookup_kpop_id(cwd, "Ma1b2c").expect("lookup");
            assert_eq!(path, exp);
        });
    }

    #[test]
    fn kpop_lookup_duplicate_ids_errors_with_two_runs() {
        with_test_home(|home, cwd| {
            for name in ["20260101_000000_runaaa01", "20260101_000000_runaaa02"] {
                let (run_dir, _) = seed_home_run(home, cwd, name);
                std::fs::write(
                    run_dir.join("stdout.log"),
                    format!(
                        "20260101.000000.000 {}KPOP_LOG: Mdup01 ./x\n",
                        format_who_tag_prefix(MALVIN_WHO)
                    ),
                )
                .expect("stdout");
            }
            let err = lookup_kpop_id(cwd, "Mdup01").unwrap_err();
            assert!(err.contains("ambiguous"), "got: {err}");
        });
    }

    #[test]
    fn kpop_lookup_not_found() {
        with_test_home(|home, cwd| {
            let bucket = home.join(".malvin/logs").join(crate::workspace_logs_hash(cwd));
            std::fs::create_dir_all(bucket).expect("mkdir");
            let err = lookup_kpop_id(cwd, "Mnope1").unwrap_err();
            assert!(err.contains("no KPOP id"), "got: {err}");
        });
    }

    #[test]
    fn kpop_lookup_reads_command_log() {
        with_test_home(|home, cwd| {
            let (run_dir, exp) = seed_home_run(home, cwd, "20260101_000000_runcmdab");
            std::fs::write(&exp, "log\n").expect("exp");
            let rel = format!(
                "{}/_kpop/exp_log_20260101_000000_runcmdab.md",
                run_dir.display()
            );
            std::fs::write(
                run_dir.join("command.log"),
                format!(
                    "20260101.000000.000 {}KPOP_LOG: Mcmd01 {rel}\n",
                    format_who_tag_prefix(MALVIN_WHO)
                ),
            )
            .expect("command.log");
            let path = lookup_kpop_id(cwd, "Mcmd01").expect("lookup");
            assert_eq!(path, exp);
        });
    }

    #[test]
    fn kpop_lookup_nested_malvin_tree() {
        with_test_home(|home, cwd| {
            let (run_dir, exp) = seed_home_run(home, cwd, "20260101_000000_innerabc");
            std::fs::write(&exp, "log\n").expect("exp");
            let rel = format!(
                "{}/_kpop/exp_log_20260101_000000_innerabc.md",
                run_dir.display()
            );
            std::fs::write(
                run_dir.join("stdout.log"),
                format!(
                    "20260101.000000.000 {}KPOP_LOG: Mnest1 {rel}\n",
                    format_who_tag_prefix(MALVIN_WHO)
                ),
            )
            .expect("stdout");
            let path = lookup_kpop_id(cwd, "Mnest1").expect("lookup");
            assert_eq!(path, exp);
        });
    }

    #[test]
    fn kpop_lookup_rejects_missing_exp_log_path() {
        with_test_home(|home, cwd| {
            let (run_dir, _) = seed_home_run(home, cwd, "20260103_000000_nopeabcd");
            std::fs::write(
                run_dir.join("stdout.log"),
                format!(
                    "20260101.000000.000 {}KPOP_LOG: Mbad01 ./missing/exp_log_x.md\n",
                    format_who_tag_prefix(MALVIN_WHO)
                ),
            )
            .expect("stdout");
            assert!(lookup_kpop_id(cwd, "Mbad01").is_err());
        });
    }
}
