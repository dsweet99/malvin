//! M (this binary) starts P in a sandbox; P spawns detached C_i and exits.
//! M waits for P via `exec`, then tears down the VM. Verify C_i die with the VM.

use microsandbox::sandbox::SandboxStatus;
use microsandbox::Sandbox;

const SANDBOX_NAME: &str = "malvin-exp-detach-children";
const CHILD_SLEEP_SECS: u32 = 300;

fn process_p_script() -> String {
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

struct ExecResult {
    exit_code: i32,
    success: bool,
    stdout: String,
}

async fn exec_run(sb: &Sandbox, cmd: &str, args: &[&str]) -> ExecResult {
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

fn count_alive_lines(stdout: &str) -> usize {
    stdout.lines().filter(|l| l.starts_with("alive pid=")).count()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("M: supervisor — start P, wait for P, tear down VM\n");

    // M: boot sandbox
    let sb = Sandbox::builder(SANDBOX_NAME)
        .image("alpine")
        .replace()
        .create()
        .await?;
    println!("M: sandbox created (status Running)");

    // M: start P and block until P exits
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

    if !p.success {
        eprintln!("VERIFY FAILED: P should exit 0");
        std::process::exit(1);
    }

    // VM still running; C_i should still be alive inside guest
    let probe = exec_run(&sb, "sh", &["-c", PROBE_GUEST]).await;
    let alive = count_alive_lines(&probe.stdout);
    println!("M: probe after P death — {alive} children alive in guest");
    if !probe.stdout.is_empty() {
        println!("{}", probe.stdout);
    }

    let handle = Sandbox::get(SANDBOX_NAME).await?;
    if handle.status() != SandboxStatus::Running {
        eprintln!(
            "VERIFY FAILED: expected VM Running after P died, got {:?}",
            handle.status()
        );
        std::process::exit(1);
    }

    if alive != 3 {
        eprintln!("VERIFY FAILED: expected 3 alive children before teardown, got {alive}");
        std::process::exit(1);
    }

    // M: tear down VM (stop only signals agentd; stop_and_wait waits for VM exit)
    println!("M: tearing down VM (stop_and_wait)...");
    let vm_exit = sb.stop_and_wait().await?;
    println!(
        "M: VM exit status code={:?} success={}",
        vm_exit.code(),
        vm_exit.success()
    );

    let handle = Sandbox::get(SANDBOX_NAME).await?;
    println!("M: sandbox status after teardown={:?}", handle.status());

    // Exec into a stopped sandbox should fail
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

    println!("\nOK: M waited for P → VM still had C_i → M tore down VM; guest unreachable.");

    Ok(())
}
