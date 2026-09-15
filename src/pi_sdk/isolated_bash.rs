use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use pi::sdk::{
    Config, Tool, ToolFactory, ToolOutput, ToolRegistry, ToolUpdate, default_tool_registry,
};
use pi::tools::ToolEffects;
use serde_json::Value;

#[path = "isolated_bash_exec.rs"]
mod isolated_bash_exec;
pub(crate) use isolated_bash_exec::{interrupt_active_isolated_bash, run_isolated_bash};

pub(crate) struct IsolatedToolFactory;

impl ToolFactory for IsolatedToolFactory {
    fn create_tool_registry(&self, enabled: &[&str], cwd: &Path, config: &Config) -> ToolRegistry {
        let registry = default_tool_registry(enabled, cwd, config);
        let tools = registry.into_tools();
        let mut replaced = Vec::with_capacity(tools.len());
        for tool in tools {
            if tool.name() == "bash" {
                replaced.push(
                    Box::new(IsolatedBash::from_builtin(tool, cwd.to_path_buf())) as Box<dyn Tool>,
                );
            } else if tool.name() == "write" {
                replaced.push(Box::new(CompleteWrite::from_builtin(tool)) as Box<dyn Tool>);
            } else {
                replaced.push(tool);
            }
        }
        ToolRegistry::from_tools(replaced)
    }
}

struct IsolatedBash {
    inner: Box<dyn Tool>,
    cwd: PathBuf,
}

impl IsolatedBash {
    fn from_builtin(inner: Box<dyn Tool>, cwd: PathBuf) -> Self {
        Self { inner, cwd }
    }
}

#[async_trait]
impl Tool for IsolatedBash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn label(&self) -> &str {
        self.inner.label()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    fn effects(&self) -> ToolEffects {
        ToolEffects::process().union(ToolEffects::write())
    }

    #[allow(clippy::unused_async)]
    async fn execute(
        &self,
        _tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let command = input
            .get("command")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let timeout_secs = input.get("timeout").and_then(Value::as_u64);
        run_isolated_bash(&self.cwd, &command, timeout_secs, on_update.as_deref())
    }
}

struct CompleteWrite {
    inner: Box<dyn Tool>,
}

impl CompleteWrite {
    fn from_builtin(inner: Box<dyn Tool>) -> Self {
        Self { inner }
    }
}

fn write_field_str<'a>(input: &'a Value, keys: &[&str]) -> &'a str {
    keys.iter()
        .find_map(|k| input.get(*k).and_then(Value::as_str))
        .unwrap_or("")
}

fn path_looks_like_bin(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    normalized.contains("/bin/") || normalized.starts_with("bin/")
}

fn stubby_write_error(input: &Value) -> Option<String> {
    let path = write_field_str(input, &["path", "file"]);
    if !path_looks_like_bin(path) {
        return None;
    }
    let trimmed = write_field_str(input, &["content", "text", "contents"]).trim();
    if trimmed.len() < 120 {
        return Some(
            "write of a bin/ script is too short; include the complete file body, not a stub"
                .to_string(),
        );
    }
    if trimmed.contains("...") && trimmed.lines().count() < 12 {
        return Some(
            "write of a bin/ script looks stubbed; include the complete file body".to_string(),
        );
    }
    None
}

fn push_escaped(out: &mut String, next: Option<char>) -> bool {
    match next {
        Some('n') => out.push('\n'),
        Some('t') => out.push('\t'),
        Some('\\') => out.push('\\'),
        Some('"') => out.push('"'),
        _ => {
            out.push('\\');
            return false;
        }
    }
    true
}

fn unescape_local_write_content(content: &str) -> String {
    if content.contains('\n') || content.contains('\r') {
        return content.to_string();
    }
    if !(content.contains("\\n") || content.contains("\\t")) {
        return content.to_string();
    }
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let next = chars.peek().copied();
            if push_escaped(&mut out, next) {
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn normalize_write_input(mut input: Value) -> Value {
    let Some(obj) = input.as_object_mut() else {
        return input;
    };
    for key in ["content", "text", "contents"] {
        if let Some(Value::String(s)) = obj.get(key).cloned() {
            let fixed = unescape_local_write_content(&s);
            if fixed != s {
                obj.insert(key.to_string(), Value::String(fixed));
            }
            break;
        }
    }
    Value::Object(obj.clone())
}

#[async_trait]
impl Tool for CompleteWrite {
    fn name(&self) -> &'static str {
        "write"
    }

    fn label(&self) -> &str {
        self.inner.label()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters(&self) -> Value {
        self.inner.parameters()
    }

    fn effects(&self) -> ToolEffects {
        self.inner.effects()
    }

    async fn execute(
        &self,
        tool_call_id: &str,
        input: Value,
        on_update: Option<Box<dyn Fn(ToolUpdate) + Send + Sync>>,
    ) -> pi::sdk::Result<ToolOutput> {
        let input = normalize_write_input(input);
        if let Some(msg) = stubby_write_error(&input) {
            return Err(pi::error::Error::validation(msg));
        }
        self.inner.execute(tool_call_id, input, on_update).await
    }
}

#[must_use]
pub(crate) fn isolated_tool_factory() -> Arc<dyn ToolFactory> {
    Arc::new(IsolatedToolFactory)
}

#[cfg(test)]
mod tests {
    use super::isolated_bash_exec::{
        interrupt_active_isolated_bash, isolated_shell, reap_isolated_shell_process_group,
        run_isolated_bash, spawn_isolated_shell, wait_isolated_output,
    };
    use std::time::{Duration, Instant};

    fn pids_matching(pattern: &str) -> Vec<u32> {
        let pgrep = std::process::Command::new("pgrep")
            .args(["-f", pattern])
            .output()
            .expect("pgrep");
        String::from_utf8_lossy(&pgrep.stdout)
            .split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect()
    }

    fn assert_no_pids_matching(pattern: &str, detail: &str) {
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            let pids = pids_matching(pattern);
            if pids.is_empty() {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "{detail}: leftover pids={pids:?} pattern={pattern:?}"
            );
            std::thread::sleep(Duration::from_millis(25));
        }
    }

    #[test]
    fn interrupt_with_no_active_shell_is_noop() {
        interrupt_active_isolated_bash();
    }

    #[test]
    fn isolated_shell_is_nonempty() {
        assert!(!isolated_shell().is_empty());
    }

    #[test]
    fn stubby_bin_write_is_rejected() {
        use super::stubby_write_error;
        use serde_json::json;
        assert!(stubby_write_error(&json!({"path": "src/x.py", "content": "x"})).is_none());
        assert!(
            stubby_write_error(&json!({
                "path": "bin/csvcut",
                "content": "#!/usr/bin/env python3\n# ... (rest of the script) ...\n"
            }))
            .is_some()
        );
        let full = format!("#!/usr/bin/env python3\n{}", "import csv\n".repeat(20));
        assert!(stubby_write_error(&json!({"path": "bin/csvcut", "content": full})).is_none());
    }

    #[test]
    fn unescape_local_write_content_fixes_literal_newlines() {
        use super::{normalize_write_input, unescape_local_write_content};
        use serde_json::json;
        let raw = "#!/usr/bin/env python3\\nimport csv\\nprint(1)\\n";
        let fixed = unescape_local_write_content(raw);
        assert!(fixed.contains('\n'));
        assert!(!fixed.contains("\\n"));
        assert_eq!(
            unescape_local_write_content("already\nhas\nlines"),
            "already\nhas\nlines"
        );
        let v = normalize_write_input(json!({"path": "bin/csvcut", "content": raw}));
        assert_eq!(v["content"].as_str().unwrap().matches('\n').count(), 3);
    }

    #[test]
    fn wait_isolated_output_drains_large_stdout_without_deadlock() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let child =
            spawn_isolated_shell(dir.path(), "python3 -c 'print(\"x\" * 131072)'").expect("spawn");
        let started = Instant::now();
        let output = wait_isolated_output(child, Some(Duration::from_secs(5))).expect("output");
        assert!(
            output.status.success(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.len() >= 131_072);
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "large stdout must not deadlock: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn run_isolated_bash_large_stderr_completes() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let output = run_isolated_bash(
            dir.path(),
            "python3 -c 'import sys; sys.stderr.write(\"y\" * 131072)'",
            Some(5),
            None,
        )
        .expect("bash");
        assert!(!output.is_error);
        let pi::model::ContentBlock::Text(text) = &output.content[0] else {
            panic!("expected text output");
        };
        assert!(text.text.len() >= 131_072);
    }

    #[test]
    fn timeout_zero_uses_default_cap() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let started = Instant::now();
        run_isolated_bash(dir.path(), "sleep 2", Some(0), None).expect("bash with timeout 0");
        assert!(
            started.elapsed() >= Duration::from_secs(2),
            "timeout 0 must use the default cap and wait for the command"
        );
        assert!(
            started.elapsed() < Duration::from_mins(2),
            "timeout 0 must not disable the wall-clock cap"
        );
    }

    #[test]
    fn kpop_timeout_positive_caps_long_sleep() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let started = Instant::now();
        let err = run_isolated_bash(dir.path(), "sleep 5", Some(2), None)
            .expect_err("timeout 2 must cap sleep 5");
        assert!(
            started.elapsed() < Duration::from_secs(4),
            "positive timeout must fail before sleep finishes: {:?}",
            started.elapsed()
        );
        let _ = err;
    }

    #[test]
    fn run_isolated_bash_reaps_background_sleep() {
        // Prefer spawn+reap over `run_isolated_bash`: a background sleep inherits the
        // piped stdout/stderr and would otherwise block pipe drainage for the full sleep.
        const MARKER: &str = "malvin_iso_bash_reap_via_run_7f3a";
        let dir = tempfile::tempdir().expect("tmpdir");
        let cmd = format!("exec -a {MARKER} sleep 30 &");
        let mut child = spawn_isolated_shell(dir.path(), &cmd).expect("spawn");
        let shell_pgid = child.id();
        std::thread::sleep(Duration::from_millis(500));
        assert!(
            child.try_wait().expect("wait").is_some(),
            "shell must exit while background sleep keeps running"
        );
        assert!(
            !pids_matching(MARKER).is_empty(),
            "precondition: marked background sleep must still be running"
        );
        reap_isolated_shell_process_group(shell_pgid);
        assert_no_pids_matching(
            MARKER,
            "background sleep must be reaped when isolated shell process group is signaled",
        );
    }

    #[test]
    fn background_sleep_not_in_affiliation_set() {
        use crate::acp::{
            clear_session_spawn_affiliation_for_test, is_session_affiliated_pid,
            note_session_affiliated_pid, refresh_session_spawn_affiliation,
        };
        const MARKER: &str = "malvin_iso_bash_affil_marker_9c2e";
        clear_session_spawn_affiliation_for_test();
        let dir = tempfile::tempdir().expect("tmpdir");
        let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
        let cmd = format!("exec -a {MARKER} sleep 30 &");
        let mut child = spawn_isolated_shell(dir.path(), &cmd).expect("spawn");
        note_session_affiliated_pid(child.id());
        std::thread::sleep(Duration::from_millis(500));
        let _ = child.try_wait().expect("wait");
        refresh_session_spawn_affiliation(None, &baseline);
        let sleep_pids = pids_matching(MARKER);
        assert!(
            !sleep_pids.is_empty(),
            "precondition: background sleep must be running before reap"
        );
        assert!(
            sleep_pids
                .iter()
                .all(|pid| !is_session_affiliated_pid(*pid)),
            "background sleep must not be in affiliation set (only shell PID was noted): {sleep_pids:?}"
        );
        reap_isolated_shell_process_group(child.id());
        clear_session_spawn_affiliation_for_test();
    }

    #[test]
    fn background_job_reaped_after_isolated_shell() {
        const MARKER: &str = "malvin_iso_bash_reap_after_shell_b41d";
        let dir = tempfile::tempdir().expect("tmpdir");
        let cmd = format!("exec -a {MARKER} sleep 30 &");
        let mut child = spawn_isolated_shell(dir.path(), &cmd).expect("spawn");
        let shell_pgid = child.id();
        std::thread::sleep(Duration::from_millis(500));
        assert!(
            child.try_wait().expect("wait").is_some(),
            "shell must exit while background sleep keeps running"
        );
        assert!(
            !pids_matching(MARKER).is_empty(),
            "precondition: marked background sleep must still be running"
        );
        reap_isolated_shell_process_group(shell_pgid);
        assert_no_pids_matching(
            MARKER,
            "background sleep must be reaped after isolated shell exits",
        );
    }
}
