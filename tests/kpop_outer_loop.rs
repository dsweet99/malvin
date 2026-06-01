//! `malvin kpop --max-loops`: outer agent loop and early exit on `## KPOP_SOLVED`.

mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux {
    use std::fs;

    use crate::common::{
        acp_mock_js, acp_mock_kpop_steps_js, exp_logs_in_run, gate_exp_logs_in_run,
        kpop_log_lines, only_run_dir, run_kpop_outer_loop, session_update_chunk_line,
    };

    #[test]
    fn kpop_max_loops_three_stops_after_first_solved() {
        let body = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const m = promptText.match(/exp_log_[^\s`]+\.md/);
    const target = m ? m[0] : null;
    const os = require('os');
    const root = path.join(os.homedir(), '.malvin', 'logs');
    if (target && fs.existsSync(root)) {
      outer: for (const hash of fs.readdirSync(root, { withFileTypes: true }).filter((e) => e.isDirectory())) {
        const bucket = path.join(root, hash.name);
        const runs = fs.readdirSync(bucket, { withFileTypes: true })
          .filter((e) => e.isDirectory()).map((e) => e.name).sort().reverse();
        for (const run of runs) {
          const p = path.join(bucket, run, '_kpop', target);
          if (fs.existsSync(p)) {
            fs.appendFileSync(p, '\n## Step 1 — KPOP mock\n## KPOP_SOLVED\n');
            break outer;
          }
        }
      }
    }";
        let mock = acp_mock_js(
            "",
            &format!(
                "{body}\n{}",
                session_update_chunk_line("agent_message_chunk", r"'done\n'")
            ),
        );
        let (out, root) = run_kpop_outer_loop(&mock, &["--max-loops", "3"]);
        assert!(out.status.success(), "kpop should succeed: {out:?}");
        assert_eq!(kpop_log_lines(&String::from_utf8_lossy(&out.stdout)).len(), 1);
        assert_eq!(
            gate_exp_logs_in_run(&only_run_dir(&root.path().join("workspace"), &root.path().join("home"))).len(),
            1
        );
    }

    #[test]
    fn kpop_max_loops_one_uses_legacy_exp_log_path() {
        let (out, root) = run_kpop_outer_loop(&acp_mock_kpop_steps_js(r"'step\n'"), &[]);
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

    #[test]
    fn kpop_max_loops_two_uses_distinct_gate_exp_logs() {
        let (out, root) = run_kpop_outer_loop(&acp_mock_kpop_steps_js(r"'step\n'"), &["--max-loops", "2"]);
        assert!(out.status.success(), "kpop should finish two outer loops: {out:?}");
        assert_eq!(kpop_log_lines(&String::from_utf8_lossy(&out.stdout)).len(), 2);
        assert_eq!(
            gate_exp_logs_in_run(&only_run_dir(&root.path().join("workspace"), &root.path().join("home"))).len(),
            2
        );
    }
}
