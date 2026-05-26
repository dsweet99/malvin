use rand::SeedableRng;
use rand::rngs::StdRng;

use super::counters::{
    agent_declared_success, block_mean_from_p_creative, count_kpop_entries, count_mbc2_entries,
    hypotheses_emitted, poisson_block_size, read_exp_log_text,
};

#[test]
fn block_mean_matches_one_minus_p_over_p() {
    assert!((block_mean_from_p_creative(0.1) - 9.0).abs() < 1e-9);
}

#[test]
fn block_mean_fallback_ten_when_mbc2_disabled() {
    assert!((block_mean_from_p_creative(0.0) - 10.0).abs() < 1e-9);
}

#[test]
fn poisson_seeded_stable() {
    let mut a = StdRng::seed_from_u64(7);
    let mut b = StdRng::seed_from_u64(7);
    assert_eq!(
        poisson_block_size(&mut a, 9.0),
        poisson_block_size(&mut b, 9.0)
    );
}

#[test]
fn poisson_draw_can_be_zero() {
    let mut rng = StdRng::seed_from_u64(2026);
    let mut saw_zero = false;
    for _ in 0..4000 {
        if poisson_block_size(&mut rng, 0.5) == 0 {
            saw_zero = true;
            break;
        }
    }
    assert!(saw_zero);
}

#[test]
fn counts_steps_in_exp_log() {
    let text = "## Step 1 — KPop x\n## Step 2 — MBC2 y\n## Step 3 — KPop z\n";
    assert_eq!(count_kpop_entries(text), 2);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 3);
}

#[test]
fn count_kpop_entries_counts_kpop_heading_from_default_prompt() {
    let text = "## Step 1 — KPOP hypothesis\n## Step 2 — KPOP second\n";
    assert_eq!(
        count_kpop_entries(text),
        2,
        "KPOP step lines from kpop_block.md must count like KPop"
    );
}

#[test]
fn count_kpop_entries_counts_lowercase_kpop_step_prefix() {
    let text = "## Step 1 — kpop hypothesis\n";
    assert_eq!(
        count_kpop_entries(text),
        1,
        "lowercase kpop step headings should count toward hypothesis budget"
    );
}

#[test]
fn count_kpop_entries_rejects_kpopulation_prefix_false_positive() {
    let text = "## Step 1 — kpopulation hypothesis\n";
    assert_eq!(
        count_kpop_entries(text),
        0,
        "words starting with kpop but not KPop steps must not count"
    );
}

#[test]
fn counts_steps_with_ascii_hyphen_separator() {
    let text = "## Step 1 - KPop x\n## Step 2 - MBC2 y\n## Step 3 - KPop z\n";
    assert_eq!(count_kpop_entries(text), 2);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 3);
}

#[test]
fn counts_steps_with_en_dash_separator() {
    let text = "## Step 1 \u{2013} KPop x\n## Step 2 \u{2013} MBC2 y\n";
    assert_eq!(count_kpop_entries(text), 1);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 2);
}

#[test]
fn counts_steps_with_mixed_dash_styles() {
    let text = "## Step 1 — KPop a\n## Step 2 - KPop b\n## Step 3 \u{2013} MBC2 c\n";
    assert_eq!(count_kpop_entries(text), 2);
    assert_eq!(count_mbc2_entries(text), 1);
    assert_eq!(hypotheses_emitted(text), 3);
}

#[test]
fn rejects_step_without_recognized_separator() {
    let text = "## Step 1 KPop no-sep\n## Step 2: KPop colon\n";
    assert_eq!(count_kpop_entries(text), 0);
    assert_eq!(hypotheses_emitted(text), 0);
}

#[test]
fn success_marker_detected() {
    assert!(!agent_declared_success("no marker"));
    assert!(agent_declared_success("## KPOP_SOLVED\ndone"));
}

#[test]
fn count_kpop_solved_markers_counts_exact_lines() {
    assert_eq!(super::count_kpop_solved_markers(""), 0);
    assert_eq!(
        super::count_kpop_solved_markers("## KPOP_SOLVED\n## KPOP_SOLVED\n"),
        2
    );
    assert_eq!(super::count_kpop_solved_markers("## KPOP_SOLVED_extra\n"), 0);
}

#[test]
fn success_marker_rejects_heading_prefix_extensions() {
    assert!(!agent_declared_success("## KPOP_SOLVED_extra\n"));
}

#[test]
fn success_marker_rejects_non_empty_remainder_after_keyword() {
    assert!(!agent_declared_success(
        "## KPOP_SOLVED  not actually solved\n"
    ));
    assert!(!agent_declared_success("## KPOP_SOLVED\tstill working\n"));
}

#[test]
fn read_exp_log_text_errors_on_missing_file() {
    let p = std::path::Path::new("/nonexistent/malvin_exp_log_read_test.md");
    let e = read_exp_log_text(p).expect_err("missing file");
    assert!(e.contains("failed to read exp log"));
}

#[test]
fn poisson_terminates_when_exp_neg_lambda_underflows_to_zero() {
    let mut rng = StdRng::seed_from_u64(42);
    for _ in 0..20 {
        let n = poisson_block_size(&mut rng, 2000.0);
        assert_ne!(n, usize::MAX);
    }
}
