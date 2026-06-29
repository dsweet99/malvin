//! Inline summarize on the last active agent when `malvin kpop --max-loops` > 1.

mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux {
    use crate::common::{
        acp_mock_kpop_steps_with_summarize_js, only_run_dir, run_kpop_outer_loop,
    };

    #[test]
    fn kpop_max_loops_one_skips_inline_summarize() {
        let mock_js = acp_mock_kpop_steps_with_summarize_js(r"'single loop\n'");
        let (out, root) = run_kpop_outer_loop(&mock_js, &["--max-loops", "1"], None);
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        let run_dir = only_run_dir(&root.path().join("workspace"), &root.path().join("home"));
        assert!(
            !run_dir.join("summary.log").exists(),
            "single-loop kpop must not run inline summarize"
        );
    }
}
