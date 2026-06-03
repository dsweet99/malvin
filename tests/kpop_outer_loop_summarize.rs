//! Summarize agent after `malvin kpop --max-loops` > 1.

mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux {
    use std::fs;

    use crate::common::{
        acp_mock_kpop_steps_with_summarize_js, only_run_dir, run_kpop_outer_loop,
    };

    #[test]
    fn kpop_max_loops_two_runs_summarize_agent() {
        let mock_js = acp_mock_kpop_steps_with_summarize_js(r"'kpop then summarize\n'");
        let (out, root) = run_kpop_outer_loop(&mock_js, &["--max-loops", "2"]);
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        let run_dir = only_run_dir(&root.path().join("workspace"), &root.path().join("home"));
        let summary_log = run_dir.join("summary.log");
        assert!(
            fs::read_to_string(&summary_log)
                .expect("read summary.log")
                .contains("SUMMARIZE_OK"),
            "expected summarize agent response in summary.log"
        );
    }

    #[test]
    fn kpop_max_loops_one_skips_summarize_agent() {
        let mock_js = acp_mock_kpop_steps_with_summarize_js(r"'single loop\n'");
        let (out, root) = run_kpop_outer_loop(&mock_js, &[]);
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        let run_dir = only_run_dir(&root.path().join("workspace"), &root.path().join("home"));
        assert!(
            !run_dir.join("summary.log").exists(),
            "single-loop kpop must not run summarize agent"
        );
    }
}
