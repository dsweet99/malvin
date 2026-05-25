# Microsandbox experimentation report

This document summarizes empirical work under `experiments/`: standalone Rust binaries that use [microsandbox](https://microsandbox.dev/) to probe VM lifecycle, memory limits, and detached child processes. The work is **not wired into malvin**; it informs whether microsandbox is a viable containment layer for runs where `cursor-agent` (or similar) spawns processes that outlive the main session.

**Host environment for runs documented here:** macOS on Apple Silicon (`arm64`), Rust 1.91.x, `microsandbox` crate **0.4.6**.

---

## Background

Malvin today spawns `cursor-agent` / `agent acp` on the **host**, with process-group isolation and explicit `shutdown()` (see `src/acp/unix_process_group.rs`, `src/acp/session_post_impl.inc`). That can still leave **orphan host processes** if the agent detaches children or if a session is dropped without teardown.

Microsandbox offers a different model: each sandbox is a **microVM** (hardware virtualization via libkrun), not a shared-kernel container. The hypothesis we tested is that a **supervisor M** on the host can:

1. Run the agent (or a stand-in **P**) inside the VM.
2. Cap guest memory.
3. When **P** exits, detect that via `exec` completion and tear down the **whole VM**, killing orphaned **C_i** inside the guest in one step.

---

## Crates and workspace layout

### Workspace

The experiments live in a **separate Cargo workspace** so they do not affect the malvin package graph or CI for the main crate.

| File | Role |
|------|------|
| [`experiments/Cargo.toml`](Cargo.toml) | Workspace root; members listed below |
| [`experiments/Cargo.lock`](Cargo.lock) | Locked dependency tree (~590 crates on first resolve, mostly via `microsandbox`) |

```1:3:experiments/Cargo.toml
[workspace]
resolver = "2"
members = ["date_in_sandbox", "memory_cap_oom", "detached_children"]
```

### Experiment binaries

Each member is a small binary (`publish = false`, `edition = "2024"`) with the same core dependency:

| Crate / binary | Path | Purpose |
|----------------|------|---------|
| `date_in_sandbox` | [`experiments/date_in_sandbox/`](date_in_sandbox/) | Smoke test: `exec("date")` inside alpine |
| `memory_cap_oom` | [`experiments/memory_cap_oom/`](memory_cap_oom/) | Guest RAM cap + OOM behavior |
| `detached_children` | [`experiments/detached_children/`](detached_children/) | P spawns detached C_i; M supervises teardown |

Per-crate manifests (all pin `microsandbox = "0.4.6"`):

- [`experiments/date_in_sandbox/Cargo.toml`](date_in_sandbox/Cargo.toml)
- [`experiments/memory_cap_oom/Cargo.toml`](memory_cap_oom/Cargo.toml)
- [`experiments/detached_children/Cargo.toml`](detached_children/Cargo.toml)

**Direct dependencies (declared):**

- `microsandbox` — SDK + embedded runtime (pulls `microsandbox-runtime`, `msb_krun`, image/OCI stack, networking, etc.)
- `tokio` — async runtime for `#[tokio::main]` and sandbox I/O

**Not depended on:** malvin, `cursor-agent`, or ACP.

### What `microsandbox` pulls in (transitive, observed)

First `cargo build` in this workspace downloads and compiles a large tree. Notable transitive families (from build logs):

- **VM:** `msb_krun`, `msb_krun_vmm`, `vm-memory`, `linux-loader` (platform-specific)
- **Runtime:** `microsandbox-runtime`, `microsandbox-protocol`, `microsandbox-image`, `microsandbox-filesystem`, `microsandbox-network`
- **Data:** `sea-orm`, `sqlx` (sandbox persistence / registry on disk)
- **OCI:** `oci-client`, image layer tooling

There is **no Docker daemon**; the runtime embeds in the experiment binary’s process. Sandboxes run as **child processes** of M.

### How to build and run

From any member directory:

```bash
cd experiments/date_in_sandbox && cargo run
cd experiments/memory_cap_oom && cargo run
cd experiments/detached_children && cargo run
```

Shared build artifacts land under `experiments/target/` (workspace target dir).

---

## Experiment 1: Basic exec (`date_in_sandbox`)

**Goal:** Confirm the toolchain boots a VM, runs a trivial command, and stops.

**Code:** [`experiments/date_in_sandbox/src/main.rs`](date_in_sandbox/src/main.rs)

Flow:

1. `Sandbox::builder("malvin-exp-date").image("alpine").replace().create().await?`
2. `sb.exec("date", []).await?` — print exit status and stdout
3. `sb.stop().await?`

```5:19:experiments/date_in_sandbox/src/main.rs
    let sb = Sandbox::builder("malvin-exp-date")
        .image("alpine")
        .replace()
        .create()
        .await?;

    let output = sb.exec("date", [] as [&str; 0]).await?;
    println!("exit={} success={}", output.status().code, output.status().success);
    println!("stdout: {}", output.stdout()?);
    // ...
    sb.stop().await?;
```

**Observed:** Success; guest prints UTC date. First run ~2–3 min (image pull + compile); subsequent runs ~30–40 s (VM boot). `replace()` avoids name collisions when re-running the same sandbox name.

---

## Memory

**Goal:** Verify that a **guest memory cap** is enforced sensibly when a process tries to exceed it.

**Code:** [`experiments/memory_cap_oom/src/main.rs`](memory_cap_oom/src/main.rs)

### Configuration

Constants and allocator script:

```8:18:experiments/memory_cap_oom/src/main.rs
const SANDBOX_NAME: &str = "malvin-exp-mem-cap";
const CAPPED_MEMORY_MIB: u32 = 48;
const CONTROL_MEMORY_MIB: u32 = 128;
const ALLOC_STEP_MIB: u32 = 4;

const ALLOCATOR_SH: &str = r#"i=0
while true; do
  i=$((i+1))
  dd if=/dev/zero of=/tmp/oom$i bs=1M count=4 2>/dev/null || exit 137
  echo "allocated $((i*4)) MiB"
done
"#;
```

Sandbox creation with cap:

```35:40:experiments/memory_cap_oom/src/main.rs
    let sb = Sandbox::builder(SANDBOX_NAME)
        .image("alpine")
        .memory(memory_mib)
        .replace()
        .create()
        .await?;
```

Verification logic expects **SIGKILL-style** failure (`exit 137`) and that reported allocation stays **below** the cap (guest kernel uses part of the budget):

```82:113:experiments/memory_cap_oom/src/main.rs
fn oom_like_exit(code: i32) -> bool {
    matches!(code, 137 | 9 | -9)
}

fn verify_capped(r: &RunReport) -> Result<(), String> {
    // ...
    if !oom_like_exit(r.exit_code) { /* ... */ }
    let alloc = last_allocated_mib(&r.stdout).ok_or_else(|| { /* ... */ })?;
    if alloc >= CAPPED_MEMORY_MIB { /* ... */ }
```

### Results (alpine + `dd` loop)

| Run | `.memory()` | Last stdout | Exit | Interpretation |
|-----|-------------|-------------|------|----------------|
| Capped | 48 MiB | `allocated 4 MiB` | **137** | OOM killer stopped the shell loop early |
| Control | 128 MiB | up to `allocated 32 MiB` | **137** | Higher cap allowed more growth before kill |

**Claim (with evidence from this experiment):** `SandboxBuilder::memory(mib)` sets a **hard guest RAM limit**. A process that allocates until pressure hits is terminated with a non-zero exit (typically 137), not left running unbounded. `sb.stop()` still works afterward.

### Python allocator (exploratory, not in current binary)

During early exploration, `python:alpine` with a 48 MiB cap and a Python heap loop often produced **`exit=-1`** and **empty stdout** — the **entire VM** died quickly, not just the Python process. The checked-in experiment uses **busybox `dd`** because it gives a clearer per-process OOM signal (progress lines + 137). For agent containment, both matter:

- **Per-process OOM (137):** predictable failure mode inside the guest.
- **VM-level death (-1):** still containment, but coarser (no partial cleanup inside the guest).

### Relation to malvin / cursor-agent

A microsandbox wrapper could cap RSS for an agent VM (e.g. 2–4 GiB) so runaway memory from leaked child **guest** processes cannot grow without bound on the host. Enforcement is at the **VM** boundary, not `kill -9` on individual host PIDs.

---

## Child processes and supervision (M, P, C_i)

**Goal:** Model host supervisor **M**, guest parent **P**, and detached children **C_i**, then answer:

- When P exits, does the VM stop? **No** (unless M stops it).
- Do C_i keep running? **Yes** (reparented to PID 1).
- Can M detect P’s death and tear down the VM so C_i die? **Yes**, if M uses the right APIs.

**Code:** [`experiments/detached_children/src/main.rs`](detached_children/src/main.rs)

### Roles

| Symbol | What it is in the experiment |
|--------|------------------------------|
| **M** | The Rust binary (`detached_children`); creates sandbox, runs `exec`, stops VM |
| **P** | Shell script run via `sb.exec("sh", ["-c", p_script])`; spawns children and exits |
| **C_i** | Three `nohup sleep 300` processes, backgrounded and disconnected from P’s terminal |

P’s script (spawn + detach + exit):

```10:26:experiments/detached_children/src/main.rs
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
```

Guest probe after P dies:

```29:41:experiments/detached_children/src/main.rs
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
// ...
"#;
```

### Phase A — P exits, VM and C_i still alive

M starts P and **blocks until P finishes** (`exec` semantics):

```77:80:experiments/detached_children/src/main.rs
    println!("M: starting P (exec waits for P to finish)...");
    let p = exec_run(&sb, "sh", &["-c", p_script.as_str()]).await;
```

After `exec` returns, M probes the guest:

```94:114:experiments/detached_children/src/main.rs
    let probe = exec_run(&sb, "sh", &["-c", PROBE_GUEST]).await;
    let alive = count_alive_lines(&probe.stdout);
    // ...
    if handle.status() != SandboxStatus::Running { /* fail */ }
    if alive != 3 { /* fail */ }
```

**Observed (representative run):**

- P exits 0; stdout lists three child PIDs.
- Probe: all three `alive pid=…`; `ps` shows `PPID 1` for each `sleep` (reparented to guest init).
- `Sandbox::get(…).status()` → `Running`.
- M can run another `exec` (probe) while C_i sleep — the VM is still up.

So: **P’s death is not VM death.** Detached C_i behave like normal orphan daemons inside the microVM.

### Phase B — M detects P and tears down the VM

**Detecting P:** In this model, P is the main `exec` session. When `exec` returns, M treats P as dead. No polling of guest PIDs is required for P itself.

**Teardown:** M must stop the **microVM process**, not only send a shutdown message:

```116:137:experiments/detached_children/src/main.rs
    println!("M: tearing down VM (stop_and_wait)...");
    let vm_exit = sb.stop_and_wait().await?;
    // ...
    let handle = Sandbox::get(SANDBOX_NAME).await?;
    println!("M: sandbox status after teardown={:?}", handle.status());

    match sb.exec("true", [] as [&str; 0]).await {
        Err(e) => println!("M: exec after teardown failed as expected: {e}"),
        Ok(o) => { /* VERIFY FAILED */ }
    }
```

**Important API distinction:**

| API | Behavior observed |
|-----|-------------------|
| `stop()` | Sends graceful shutdown to guest agent; **does not** wait for VM exit. Status can remain `Running` for a long time. |
| `stop_and_wait()` | Shutdown + wait for VM process exit; status becomes `Stopped`; further `exec` fails (e.g. broken pipe). |

**Observed after `stop_and_wait()`:**

- `SandboxStatus::Stopped`
- `exec("true")` → runtime error (`Broken pipe`)
- C_i are gone with the VM (no separate host orphan PIDs for those sleeps)

### Attached vs detached lifecycle (host)

`Sandbox::create()` in these experiments uses default **attached** lifecycle (`owns_lifecycle() == true`): the VM is tied to M’s ownership of the `Sandbox` handle. That is separate from guest P exiting:

- **Guest P exits** → VM keeps running; C_i can survive.
- **M calls `stop_and_wait()`** → VM exits; all guest processes die.
- **`create_detached()`** (not used here) — VM can survive after the **host** process exits; see [microsandbox lifecycle docs](https://docs.microsandbox.dev/sandboxes/lifecycle).

### Intended malvin mapping

| Experiment | Possible malvin role |
|------------|----------------------|
| M | Malvin (or a thin runner) spawning one sandbox per agent session |
| P | `cursor-agent` / `agent acp` main process inside the VM |
| C_i | Dev servers, watchers, subprocesses the agent leaves behind |
| `exec` return | End of primary agent RPC/session (or explicit cancel) |
| `stop_and_wait()` | Session end policy: always destroy VM so C_i cannot leak |

Malvin’s current host-side `terminate_process_group` + `shutdown()` ([`src/acp/unix_process_group.rs`](../src/acp/unix_process_group.rs), [`src/acp/session_post_impl.inc`](../src/acp/session_post_impl.inc)) targets **host** PGIDs; microsandbox would add **VM-level** cleanup. Note: `AcpSession`’s `Drop` is currently a no-op ([`session_post_impl.inc`](../src/acp/session_post_impl.inc)), so host sessions still require explicit `shutdown()` — any sandbox supervisor would need the same discipline.

---

## Summary table

| Topic | Finding |
|-------|---------|
| **Crates** | Isolated workspace; `microsandbox 0.4.6` + `tokio`; large transitive VM/OCI stack; no malvin link |
| **Memory** | `.memory(mib)` enforces cap; alpine `dd` allocator gets SIGKILL (137); user-allocation progress stops well below nominal cap due to guest OS overhead |
| **Child processes** | Detached C_i survive P; reparent to PID 1; VM stays `Running` |
| **Supervision** | M learns P died when `exec` returns; M must `stop_and_wait()` to kill C_i and release the VM; `stop()` alone is insufficient |

---

## Open questions (not tested here)

- Running real `cursor-agent` inside a sandbox (binary in image, ACP stdio, auth, workspace mounts).
- Performance vs host-native ACP (boot ~tens of seconds per session in these runs).
- `create_detached()` for long-lived VMs vs per-session attached VMs.
- Idle timeouts (`idle_timeout_secs`, `max_duration_secs` on builder) as a backstop if M crashes without `stop_and_wait()`.
- Interaction with Cursor’s own Landlock/seccomp sandbox (`--sandbox enabled` on host agent) — orthogonal layer.

---

## References

- [microsandbox.dev](https://microsandbox.dev/)
- [Docs index](https://docs.microsandbox.dev/llms.txt)
- [Quickstart](https://docs.microsandbox.dev/getting-started/quickstart)
- [Sandbox overview (memory, replace, grace)](https://docs.microsandbox.dev/sandboxes/overview)
- [Cursor agent sandboxing (host)](https://cursor.com/blog/agent-sandboxing)
