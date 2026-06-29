//! `malvin kpop --max-loops`: outer agent loop and early exit on `## KPOP_SOLVED`.

mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux {
    use std::fs;

    use crate::common::{
        acp_mock_kpop_steps_js, acp_mock_kpop_writes_solved_js, exp_logs_in_run,
        gate_exp_logs_in_run, kpop_log_lines, only_run_dir, run_kpop_outer_loop,
    };

    #[test]
    fn kpop_max_loops_three_stops_after_first_solved() {
        let mock = acp_mock_kpop_writes_solved_js(r"'done\n'");
        let (out, root) = run_kpop_outer_loop(&mock, &["--max-loops", "3"], Some("mpc = false\n"));
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        assert_eq!(kpop_log_lines(&String::from_utf8_lossy(&out.stdout)).len(), 1);
        assert_eq!(
            gate_exp_logs_in_run(&only_run_dir(&root.path().join("workspace"), &root.path().join("home"))).len(),
            1
        );
    }

    #[test]
    fn kpop_max_loops_one_uses_legacy_exp_log_path() {
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
