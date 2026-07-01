//! `malvin kpop --max-loops`: outer agent loop with `KPop` implementer sessions.

mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux {
    use std::fs;

    use crate::common::{
        acp_mock_kpop_steps_js, exp_logs_in_run, gate_exp_logs_in_run, only_run_dir,
        run_kpop_outer_loop,
    };

    #[test]
    fn kpop_max_loops_one_writes_legacy_exp_log() {
        let (out, root) = run_kpop_outer_loop(&acp_mock_kpop_steps_js(r"'step\n'"), &["--max-loops", "1"], None);
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        let run_dir = only_run_dir(&root.path().join("workspace"), &root.path().join("home"));
        let legacy = exp_logs_in_run(&run_dir)
            .into_iter()
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| !n.contains("_g"))
            })
            .expect("legacy exp log");
        assert!(fs::read_to_string(&legacy).expect("read").contains("## Step 1 — KPOP mock"));
        assert!(gate_exp_logs_in_run(&run_dir).is_empty());
    }
}
