//! Integration-style exercises for multiturn KPOP state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

mod common;

use common::{append_kpop_line, parse_kpop_want, MtStubPrompts, MBC2_SEEK_MAX_STEPS};
use malvin::MultiturnPrompt;
use malvin::kpop_progression::{KpopMultiturnParams, KpopMultiturnState};
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{KPOP_CATCHUP_CAP, hypotheses_emitted};
use rand::SeedableRng;
use rand::rngs::StdRng;

struct EchoPrompts;

impl KpopMultiturnPrompts for EchoPrompts {
    fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        Ok(format!("K{want}"))
    }

    fn mbc2_pure(&mut self) -> Result<String, String> {
        Ok("M".into())
    }
}

#[test]
fn multiturn_stops_immediately_when_exp_log_already_at_max_hypotheses() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(
        &path,
        "## Step 1 — KPOP a\n## Step 2 — KPOP b\n## Step 3 — MBC2 c\n",
    )
    .unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: EchoPrompts,
        exp_log_path: path,
        max_hypotheses: 3,
        p_creative: 0.5,
        rng: StdRng::seed_from_u64(1),
    })
    .unwrap();
    assert!(state.next_prompt().unwrap().is_none());
}

#[test]
fn multiturn_exits_when_exp_log_hits_success_marker() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), "## KPOP_SOLVED\nx\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: EchoPrompts,
        exp_log_path: tmp.path().to_path_buf(),
        max_hypotheses: 100,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(1),
    })
    .unwrap();
    assert!(state.next_prompt().unwrap().is_none());
}

#[test]
fn kpop_want_respects_global_max_hypotheses() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: MtStubPrompts,
        exp_log_path: path,
        max_hypotheses: 3,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(42),
    })
    .unwrap();
    let first = state.next_prompt().expect("prompt").expect("first");
    let MultiturnPrompt::KpopBlock(s) = first else {
        panic!("expected kpop block");
    };
    let want = parse_kpop_want(&s).expect("want");
    assert!(want <= 3, "want={want} must not exceed max_hypotheses");
}

#[test]
fn kpop_block_finishes_after_agent_writes_enough_steps() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: MtStubPrompts,
        exp_log_path: path.clone(),
        max_hypotheses: 50,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(42),
    })
    .unwrap();
    let first = state.next_prompt().expect("prompt");
    let MultiturnPrompt::KpopBlock(s) = first.expect("first") else {
        panic!("expected kpop block");
    };
    let want = parse_kpop_want(&s).expect("want in stub");
    for step in 1..=want {
        append_kpop_line(&path, step);
    }
    let p2 = state.next_prompt().expect("second");
    assert!(p2.is_some());
    assert!(hypotheses_emitted(&std::fs::read_to_string(&path).unwrap()) >= want);
}

#[test]
fn kpop_catch_up_exhausted_returns_error_when_log_stays_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: MtStubPrompts,
        exp_log_path: path,
        max_hypotheses: 100,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(99),
    })
    .unwrap();
    for _ in 0..=KPOP_CATCHUP_CAP {
        assert!(state.next_prompt().unwrap().is_some());
        state.record_kpop_block_prompt_completed();
    }
    let err = state
        .next_prompt()
        .expect_err("expected catch-up exhaustion");
    assert!(
        err.contains("initial attempt") && err.contains("catch-up attempts"),
        "unexpected error: {err}"
    );
}

#[test]
fn mbc2_without_dispatch_record_reissues_first_prompt() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: MtStubPrompts,
        exp_log_path: path.clone(),
        max_hypotheses: 20,
        p_creative: 0.5,
        rng: StdRng::seed_from_u64(7),
    })
    .unwrap();
    let mut step = 1usize;
    for _ in 0..MBC2_SEEK_MAX_STEPS {
        let p = state.next_prompt().expect("prompt");
        let Some(pr) = p else {
            panic!("unexpected stop");
        };
        let s = pr.as_str();
        if s == "stub mbc2" {
            let again = state.next_prompt().expect("again").unwrap();
            assert_eq!(again.as_str(), "stub mbc2");
            return;
        }
        let w = parse_kpop_want(s).expect("kpop");
        for _ in 0..w {
            append_kpop_line(&path, step);
            step += 1;
        }
    }
    panic!("expected stub mbc2 within {MBC2_SEEK_MAX_STEPS} scheduler steps");
}
