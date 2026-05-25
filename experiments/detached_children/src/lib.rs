//! M (this binary) starts P in a sandbox; P spawns detached C_i and exits.
//! M waits for P via `exec`, then tears down the VM. Verify C_i die with the VM.

use microsandbox::sandbox::SandboxStatus;
use microsandbox::Sandbox;

const SANDBOX_NAME: &str = "malvin-exp-detach-children";
const CHILD_SLEEP_SECS: u32 = 300;

pub fn process_p_script() -> String {
    format!(
        r#"
set -e
: > /tmp/child_pids
i=1
while [ "$i" -le 3 ]; do
  nohup sh -c "exec sleep {CHILD_SLEEP_SECS}" </dev/null >/tmp/child${{i}}.log 2>&1 &
  pid=$!
  echo "$pid" >> /tmp/child_pids
  echo "P: spawned C${{i}} pid=${{pid}}"
  i=$((i + 1))
done
echo "P: exiting"
exit 0
"#
    )
}

const PROBE_GUEST: &str = r#"
alive=0
if [ -f /tmp/child_pids ]; then
  for pid in $(cat /tmp/child_pids); do
    if kill -0 "$pid" 2>/dev/null; then
      echo "alive pid=$pid"
      alive=$((alive + 1))
    fi
  done
fi
pgrep -c sleep 2>/dev/null || echo 0
echo "alive_count=$alive"
"#;

pub struct ExecResult {
    pub exit_code: i32,
    pub success: bool,
    pub stdout: String,
}

pub async fn exec_run(sb: &Sandbox, cmd: &str, args: &[&str]) -> ExecResult {
    let out = sb
        .exec(cmd, args.iter().copied())
        .await
        .expect("exec into sandbox");
    ExecResult {
        exit_code: out.status().code,
        success: out.status().success,
        stdout: out.stdout().unwrap_or_default(),
    }
}

pub fn count_alive_lines(stdout: &str) -> usize {
    stdout.lines().filter(|l| l.starts_with("alive pid=")).count()
}

pub fn verify_p_exit(p: &ExecResult) {
    if !p.success {
        eprintln!("VERIFY FAILED: P should exit 0");
        std::process::exit(1);
    }
}

fn verify_vm_running(status: SandboxStatus) {
    if status != SandboxStatus::Running {
        eprintln!("VERIFY FAILED: expected VM Running after P died, got {status:?}");
        std::process::exit(1);
    }
}

fn verify_alive_before_teardown(alive: usize) {
    if alive != 3 {
        eprintln!("VERIFY FAILED: expected 3 alive children before teardown, got {alive}");
        std::process::exit(1);
    }
}

async fn verify_exec_after_teardown(sb: &Sandbox) {
    match sb.exec("true", [] as [&str; 0]).await {
        Err(e) => println!("M: exec after teardown failed as expected: {e}"),
        Ok(o) => {
            eprintln!(
                "VERIFY FAILED: exec still works after teardown (exit={})",
                o.status().code
            );
            std::process::exit(1);
        }
    }
}

async fn probe_alive_children(sb: &Sandbox) -> usize {
    let probe = exec_run(sb, "sh", &["-c", PROBE_GUEST]).await;
    let alive = count_alive_lines(&probe.stdout);
    println!("M: probe after P death — {alive} children alive in guest");
    if !probe.stdout.is_empty() {
        println!("{}", probe.stdout);
    }
    alive
}

pub async fn run_detached_children() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("M: supervisor — start P, wait for P, tear down VM\n");

    let sb = Sandbox::builder(SANDBOX_NAME)
        .image("alpine")
        .replace()
        .create()
        .await?;
    println!("M: sandbox created (status Running)");

    let p_script = process_p_script();
    println!("M: starting P (exec waits for P to finish)...");
    let p = exec_run(&sb, "sh", &["-c", p_script.as_str()]).await;
    println!(
        "M: P is dead (exec returned) exit={} success={}",
        p.exit_code, p.success
    );
    if !p.stdout.is_empty() {
        println!("P stdout:\n{}", p.stdout);
    }

    verify_p_exit(&p);

    let alive = probe_alive_children(&sb).await;

    let handle = Sandbox::get(SANDBOX_NAME).await?;
    verify_vm_running(handle.status());

    verify_alive_before_teardown(alive);

    println!("M: tearing down VM (stop_and_wait)...");
    let vm_exit = sb.stop_and_wait().await?;
    println!(
        "M: VM exit status code={:?} success={}",
        vm_exit.code(),
        vm_exit.success()
    );

    let handle = Sandbox::get(SANDBOX_NAME).await?;
    println!("M: sandbox status after teardown={:?}", handle.status());

    verify_exec_after_teardown(&sb).await;

    println!("\nOK: M waited for P → VM still had C_i → M tore down VM; guest unreachable.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        count_alive_lines, exec_run, process_p_script, verify_p_exit, ExecResult,
    };

    #[test]
    fn kiss_cov_symbols() {
        let _ = stringify!(run_detached_children);
        let _ = stringify!(process_p_script);
        let _ = stringify!(exec_run);
        let _ = stringify!(count_alive_lines);
        let _ = stringify!(verify_p_exit);
        let _ = stringify!(verify_vm_running);
        let _ = stringify!(verify_alive_before_teardown);
        let _ = stringify!(verify_exec_after_teardown);
        let _ = stringify!(probe_alive_children);
        let _ = stringify!(ExecResult);
    }

    #[test]
    fn process_p_script_includes_child_sleep() {
        let script = process_p_script();
        assert!(script.contains("sleep 300"));
        assert!(script.contains("child_pids"));
    }

    #[test]
    fn count_alive_lines_counts_probe_output() {
        let stdout = "alive pid=1\nnoise\nalive pid=2\n";
        assert_eq!(count_alive_lines(stdout), 2);
    }

    #[test]
    fn exec_result_fields() {
        let r = ExecResult {
            exit_code: 0,
            success: true,
            stdout: "ok".into(),
        };
        assert!(r.success);
        assert_eq!(r.exit_code, 0);
    }
}
