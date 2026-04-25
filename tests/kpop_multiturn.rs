//! Integration-style exercises for multiturn KPOP state: simulate the agent by appending to the exp log
//! between `next_prompt` calls (no real `agent acp` child).

use std::cell::RefCell;
use std::rc::Rc;

use malvin::kpop_multiturn::{KpopMultiturnParams, KpopMultiturnState};
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_schedule::{
    KPOP_CATCHUP_CAP, block_mean_from_p_creative, count_mbc2_entries, hypotheses_emitted,
    poisson_block_size,
};
use malvin::MultiturnPrompt;
use rand::SeedableRng;
use rand::rngs::StdRng;

const MBC2_SEEK_MAX_STEPS: usize = 10_000;

struct StubPrompts;

impl KpopMultiturnPrompts for StubPrompts {
    fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        Ok(format!("stub kpop want={want}"))
    }

    fn mbc2_pure(&mut self) -> Result<String, String> {
        Ok("stub mbc2".into())
    }
}

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

fn parse_kpop_want(prompt: &str) -> Option<usize> {
    prompt
        .trim()
        .strip_prefix("stub kpop want=")
        .and_then(|s| s.parse().ok())
}

fn append_kpop_line(path: &std::path::Path, step: usize) {
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

fn append_mbc2_line(path: &std::path::Path, step: usize) {
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

#[test]
fn kpop_want_respects_global_max_hypotheses() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: StubPrompts,
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
        builder: StubPrompts,
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
    assert!(
        hypotheses_emitted(&std::fs::read_to_string(&path).unwrap()) >= want
    );
}

#[test]
fn kpop_catch_up_exhausted_returns_error_when_log_stays_empty() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: StubPrompts,
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
    let err = state.next_prompt().expect_err("expected catch-up exhaustion");
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
        builder: StubPrompts,
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

#[test]
fn mbc2_pure_retries_once_when_no_new_mbc2_line() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: StubPrompts,
        exp_log_path: path.clone(),
        max_hypotheses: 20,
        p_creative: 0.5,
        rng: StdRng::seed_from_u64(7),
    })
    .unwrap();
    let mut step = 1usize;
    let mut saw_first_mbc2 = false;
    for _ in 0..128 {
        let p = state.next_prompt().expect("prompt");
        let Some(pr) = p else {
            panic!("unexpected stop before MBC2");
        };
        let s = pr.as_str();
        if s == "stub mbc2" {
            saw_first_mbc2 = true;
            state.record_mbc2_prompt_completed();
            break;
        }
        let w = parse_kpop_want(s).expect("kpop or mbc2");
        for _ in 0..w {
            append_kpop_line(&path, step);
            step += 1;
        }
    }
    assert!(saw_first_mbc2);
    assert_eq!(
        count_mbc2_entries(&std::fs::read_to_string(&path).unwrap()),
        0
    );
    let mbc2_retry = state.next_prompt().expect("mbc2 retry");
    assert!(matches!(
        mbc2_retry,
        Some(MultiturnPrompt::Mbc2(ref s)) if s == "stub mbc2"
    ));
    append_mbc2_line(&path, 1);
    let _ = state.next_prompt().expect("after mbc2 line");
    assert_eq!(
        count_mbc2_entries(&std::fs::read_to_string(&path).unwrap()),
        1
    );
}

#[test]
fn kpop_solved_stops_before_mbc2_when_creative_enabled() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(
        &path,
        "## Step 1 — KPOP test\n## KPOP_SOLVED\ndone\n",
    )
    .unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: StubPrompts,
        exp_log_path: path,
        max_hypotheses: 50,
        p_creative: 0.5,
        rng: StdRng::seed_from_u64(100),
    })
    .unwrap();
    assert!(state.next_prompt().expect("after solved").is_none());
}

struct CaptureWants {
    wants: Rc<RefCell<Vec<usize>>>,
}

impl KpopMultiturnPrompts for CaptureWants {
    fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        self.wants.borrow_mut().push(want);
        Ok(format!("stub kpop want={want}"))
    }

    fn mbc2_pure(&mut self) -> Result<String, String> {
        Ok("stub mbc2".into())
    }
}

#[test]
fn overshoot_credit_adds_to_next_block_target() {
    let mean = block_mean_from_p_creative(0.0);
    let mut rng = StdRng::seed_from_u64(42);
    let tn1 = poisson_block_size(&mut rng, mean).max(1);
    let overshoot = 3usize;
    let expected_n2 = (overshoot + poisson_block_size(&mut rng, mean)).max(1);

    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let wants = Rc::new(RefCell::new(Vec::new()));
    let cap = CaptureWants {
        wants: Rc::clone(&wants),
    };
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: cap,
        exp_log_path: path.clone(),
        max_hypotheses: 1000,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(42),
    })
    .unwrap();

    let p1 = state.next_prompt().unwrap().expect("first prompt");
    let MultiturnPrompt::KpopBlock(s1) = p1 else {
        panic!("expected kpop block");
    };
    let got1: usize = s1
        .trim()
        .strip_prefix("stub kpop want=")
        .and_then(|x| x.parse().ok())
        .expect("want");
    assert_eq!(got1, tn1);

    for step in 1..=(tn1 + overshoot) {
        append_kpop_line(&path, step);
    }

    let p2 = state.next_prompt().unwrap().expect("second prompt");
    let MultiturnPrompt::KpopBlock(s2) = p2 else {
        panic!("expected kpop block");
    };
    let got2: usize = s2
        .trim()
        .strip_prefix("stub kpop want=")
        .and_then(|x| x.parse().ok())
        .expect("want");
    assert_eq!(got2, expected_n2);

    assert_eq!(&*wants.borrow(), &[tn1, expected_n2]);
}
