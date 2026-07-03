pub use malvin::MtStubPrompts;

use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};
use std::path::PathBuf;

pub struct MultiturnTestHarness<'a> {
    pub state: KpopMultiturnState<'a>,
    pub exp_path: PathBuf,
    pub _tmp: tempfile::TempDir,
}

pub fn setup_multiturn_stub_mt() -> MultiturnTestHarness<'static> {
    let tmp = tempfile::tempdir().unwrap();
    let exp_path = tmp.path().join("exp.md");
    std::fs::write(&exp_path, "").unwrap();
    let mpc_plan = tmp.path().join("mpc_plan.md");
    let state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: KpopMultiturnPrompts::StubMt(MtStubPrompts),
        exp_log_path: exp_path.clone(),
        mpc_plan_path: mpc_plan,
    })
    .unwrap();
    MultiturnTestHarness { state, exp_path, _tmp: tmp }
}

pub const MBC2_SEEK_MAX_STEPS: usize = 10_000;

pub fn parse_kpop_want(prompt: &str) -> Option<usize> {
    prompt
        .trim()
        .strip_prefix("stub kpop want=")
        .and_then(|s| s.parse().ok())
}

pub fn append_kpop_line(path: &std::path::Path, step: usize) {
    let line = format!("## Step {step} — KPOP test\n");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        })
        .expect("append kpop");
}

pub fn append_mbc2_line(path: &std::path::Path, step: usize) {
    let line = format!("## Step {step} — MBC2 test\n");
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| {
            use std::io::Write;
            f.write_all(line.as_bytes())
        })
        .expect("append mbc2");
}
