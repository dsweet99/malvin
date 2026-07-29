use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::kpop_log_line;
    use std::path::PathBuf;

    #[test]
    fn kpop_log_line_formats_relative_exp_path() {
        let work = PathBuf::from("/tmp/work");
        let run = PathBuf::from("/tmp/work/.malvin_home/logs/h/run");
        let exp = run.join("_kpop/exp_log_x.md");
        let line = kpop_log_line("abc12", &work, &run, &exp);
        assert!(line.starts_with("KPOP_LOG: abc12 "));
        assert!(line.contains("exp_log_x.md"));
    }

    #[test]
    fn kiss_cov_kpop_log_line() {
        let _ = kpop_log_line;
    }
}
