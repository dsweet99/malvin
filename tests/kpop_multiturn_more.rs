mod common;

use std::cell::RefCell;
use std::rc::Rc;

use common::{
    append_kpop_line, append_mbc2_line, parse_kpop_want, MtStubPrompts,
};
use malvin::MultiturnPrompt;
use malvin::kpop_multiturn_prompts::KpopMultiturnPrompts;
use malvin::kpop_progression::{
    block_mean_from_p_creative, count_mbc2_entries, poisson_block_size, KpopMultiturnParams,
    KpopMultiturnState,
};
use rand::SeedableRng;
use rand::rngs::StdRng;

fn advance_until_first_mbc2_hit(
    state: &mut KpopMultiturnState<MtStubPrompts>,
    path: &std::path::Path,
) -> bool {
    let mut step = 1usize;
    for _ in 0..128 {
        let p = state.next_prompt().expect("prompt");
        let Some(pr) = p else {
            return false;
        };
        let s = pr.as_str();
        if s == "stub mbc2" {
            state.record_mbc2_prompt_completed();
            return true;
        }
        let w = parse_kpop_want(s).expect("kpop or mbc2");
        for _ in 0..w {
            append_kpop_line(path, step);
            step += 1;
        }
    }
    false
}

#[test]
fn mbc2_pure_retries_once_when_no_new_mbc2_line() {
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
    assert!(advance_until_first_mbc2_hit(&mut state, &path));
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
    std::fs::write(&path, "## Step 1 — KPOP test\n## KPOP_SOLVED\ndone\n").unwrap();
    let mut state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: MtStubPrompts,
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

fn overshoot_pair_sizes(rng_seed: u64) -> (usize, usize, usize) {
    let mean = block_mean_from_p_creative(0.0);
    let mut rng = StdRng::seed_from_u64(rng_seed);
    let tn1 = poisson_block_size(&mut rng, mean).max(1);
    let overshoot = 3usize;
    let expected_n2 = (overshoot + poisson_block_size(&mut rng, mean)).max(1);
    (tn1, overshoot, expected_n2)
}

fn parse_stub_kpop_want(s: &str) -> usize {
    s.trim()
        .strip_prefix("stub kpop want=")
        .and_then(|x| x.parse().ok())
        .expect("want")
}

fn append_kpop_steps(path: &std::path::Path, from: usize, to: usize) {
    for step in from..=to {
        append_kpop_line(path, step);
    }
}

struct OvershootCtx {
    _tmp: tempfile::TempDir,
    path: std::path::PathBuf,
    state: KpopMultiturnState<CaptureWants>,
    tn1: usize,
    overshoot: usize,
    expected_n2: usize,
    wants: Rc<RefCell<Vec<usize>>>,
}

fn overshoot_open(seed: u64) -> OvershootCtx {
    let (tn1, overshoot, expected_n2) = overshoot_pair_sizes(seed);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "").unwrap();
    let wants = Rc::new(RefCell::new(Vec::new()));
    let cap = CaptureWants {
        wants: Rc::clone(&wants),
    };
    let state = KpopMultiturnState::from_params(KpopMultiturnParams {
        builder: cap,
        exp_log_path: path.clone(),
        max_hypotheses: 1000,
        p_creative: 0.0,
        rng: StdRng::seed_from_u64(seed),
    })
    .unwrap();
    OvershootCtx {
        _tmp: tmp,
        path,
        state,
        tn1,
        overshoot,
        expected_n2,
        wants,
    }
}

fn overshoot_assert_first(ctx: &mut OvershootCtx) {
    let p1 = ctx.state.next_prompt().unwrap().expect("first prompt");
    let MultiturnPrompt::KpopBlock(s1) = p1 else {
        panic!("expected kpop block");
    };
    assert_eq!(parse_stub_kpop_want(&s1), ctx.tn1);
}

fn overshoot_assert_second(ctx: &mut OvershootCtx) {
    append_kpop_steps(&ctx.path, 1, ctx.tn1 + ctx.overshoot);
    let p2 = ctx.state.next_prompt().unwrap().expect("second prompt");
    let MultiturnPrompt::KpopBlock(s2) = p2 else {
        panic!("expected kpop block");
    };
    assert_eq!(parse_stub_kpop_want(&s2), ctx.expected_n2);
}

fn overshoot_two_block_wants(seed: u64) -> (usize, usize, Vec<usize>) {
    let mut ctx = overshoot_open(seed);
    overshoot_assert_first(&mut ctx);
    overshoot_assert_second(&mut ctx);
    (
        ctx.tn1,
        ctx.expected_n2,
        ctx.wants.borrow().to_vec(),
    )
}

#[test]
fn overshoot_credit_adds_to_next_block_target() {
    let (tn1, expected_n2, got) = overshoot_two_block_wants(42);
    assert_eq!(got, vec![tn1, expected_n2]);
}
