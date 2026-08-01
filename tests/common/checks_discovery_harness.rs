use std::path::Path;
use std::process::Command;

use super::{activate_test_home, cached_mock_executable, INTEGRATION_TEST_MALVIN_ARGS};
use super::acp_tidy_kpop::{
    acp_mock_kpop_block_match_js, acp_mock_kpop_iteration_body, acp_mock_kpop_prompt_preamble,
};

fn acp_mock_combined_discovery_and_kpop_js(kpop_chunk: &str) -> String {
    let discovery = r"    if (promptText.includes('Discover how the repo in')) {
      function resolvePromptPath(relOrAbs) {
        if (relOrAbs.startsWith('./')) return path.join(process.cwd(), relOrAbs.slice(2));
        if (relOrAbs.startsWith('/')) return relOrAbs;
        return path.join(process.cwd(), relOrAbs);
      }
      const pathMatch = promptText.match(/([^\s`]+\/_kpop\/exp_log_[^\s`]+\.md)/);
      if (pathMatch) {
        const expPath = resolvePromptPath(pathMatch[1]);
        fs.mkdirSync(path.dirname(expPath), { recursive: true });
        fs.appendFileSync(expPath, '\n## Step 1 — KPOP mock\n');
      }
      const checksPath = path.join(process.cwd(), '.malvin', 'checks');
      fs.mkdirSync(path.dirname(checksPath), { recursive: true });
      fs.writeFileSync(checksPath, 'make lint\n');
    }";
    let kpop_done = super::acp_core::session_update_chunk_line(
        "agent_message_chunk",
        kpop_chunk,
    );
    let body = format!(
        "{}\n{discovery}\n    else if ({}) {{\n{}\n{kpop_done}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_block_match_js(),
        acp_mock_kpop_iteration_body(),
    );
    super::acp_core::acp_mock_js("", &body)
}

pub fn acp_mock_checks_discovery_and_code_js() -> String {
    acp_mock_combined_discovery_and_kpop_js(r"'code kpop step\n'")
}

pub fn acp_mock_checks_discovery_no_write_js() -> String {
    let body = format!(
        "{}\n    if (promptText.includes('Discover how the repo in')) {{\n      const pathMatch = promptText.match(/([^\\s`]+\\/_kpop\\/exp_log_[^\\s`]+\\.md)/);\n      if (pathMatch) {{\n        let p = pathMatch[1];\n        let expPath = p.startsWith('./') ? path.join(process.cwd(), p.slice(2)) : path.join(process.cwd(), p);\n        fs.mkdirSync(path.dirname(expPath), {{ recursive: true }});\n        fs.appendFileSync(expPath, '\\n## Step 1 — KPOP mock\\n');\n      }}\n    }}",
        acp_mock_kpop_prompt_preamble(),
    );
    super::acp_core::acp_mock_js("", &body)
}

pub struct CodeDiscoverySpawn<'a> {
    pub project: &'a Path,
    pub home: &'a Path,
    pub mock_js: &'a str,
    pub path_var: &'a str,
    pub request: &'a str,
}

pub fn spawn_malvin_code_discovery(c: &CodeDiscoverySpawn<'_>) -> std::process::Output {
    activate_test_home(c.home);
    let mock_bin = cached_mock_executable(c.mock_js);
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(c.project)
        .env("HOME", c.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_TEST_NO_REAL_AGENT", "1")
        .env("MALLOC_ARENA_MAX", "2")
        .env("MALVIN_AGENT_ACP_BIN", mock_bin.as_os_str())
        .env("PATH", c.path_var);
    cmd.args(["code"]);
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.args(["--trust-the-plan", "--max-loops", "0"]);
    cmd.arg(c.request);
    cmd.output().expect("spawn malvin code")
}

fn count_run_dirs(bucket: &Path) -> usize {
    std::fs::read_dir(bucket)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .count()
        })
        .unwrap_or(0)
}

pub fn count_malvin_run_dirs(workspace: &Path, home: &Path) -> usize {
    count_run_dirs(&super::malvin_run_logs_bucket(workspace, home))
}
