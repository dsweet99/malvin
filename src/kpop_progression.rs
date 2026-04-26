use std::path::Path;

use rand::Rng;

pub const KPOP_CATCHUP_CAP: u32 = 3;

#[must_use]
pub fn block_mean_from_p_creative(p_creative: f64) -> f64 {
    if crate::kpop_acp_prompt::kpop_creative_enabled(p_creative) {
        let p = p_creative.clamp(0.0, 1.0);
        ((1.0 - p) / p).max(1.0)
    } else {
        10.0
    }
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn poisson_large_mean_normal_approx(rng: &mut impl Rng, lambda: f64) -> usize {
    let u1 = f64::max(f64::MIN_POSITIVE, rng.r#gen::<f64>());
    let u2 = rng.r#gen::<f64>();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    let raw = (lambda + z * lambda.sqrt()).round();
    if raw <= 0.0 {
        return 0;
    }
    if raw > usize::MAX as f64 {
        return usize::MAX;
    }
    raw as usize
}

#[must_use]
pub fn poisson_block_size(rng: &mut impl Rng, mean: f64) -> usize {
    let lambda = mean.max(1e-12);
    let l = (-lambda).exp();
    if l == 0.0 {
        return poisson_large_mean_normal_approx(rng, lambda);
    }
    let mut k = 0_usize;
    let mut p = 1.0_f64;
    loop {
        k += 1;
        p *= rng.r#gen::<f64>();
        if p <= l {
            return k - 1;
        }
    }
}

pub fn read_exp_log_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read exp log {}: {e}", path.display()))
}

fn step_kind(line: &str) -> Option<&'static str> {
    let t = line.trim_start();
    let rest = t.strip_prefix("## Step ")?;
    let tail = [" — ", " – ", " - "]
        .iter()
        .find_map(|sep| rest.split_once(sep).map(|(_, t)| t))?;
    let tail = tail.trim_start();
    if tail.starts_with("KPOP") {
        return Some("KPOP");
    }
    if tail.starts_with("MBC2") {
        return Some("MBC2");
    }
    None
}

#[must_use]
pub fn count_kpop_entries(text: &str) -> usize {
    text.lines()
        .filter(|line| step_kind(line) == Some("KPOP"))
        .count()
}

#[must_use]
pub fn count_mbc2_entries(text: &str) -> usize {
    text.lines()
        .filter(|line| step_kind(line) == Some("MBC2"))
        .count()
}

#[must_use]
pub fn hypotheses_emitted(text: &str) -> usize {
    count_kpop_entries(text) + count_mbc2_entries(text)
}

#[must_use]
pub fn agent_declared_success(text: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("## KPOP_SOLVED") else {
            return false;
        };
        rest.trim().is_empty()
    })
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{
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
        let text = "## Step 1 — KPOP x\n## Step 2 — MBC2 y\n## Step 3 — KPOP z\n";
        assert_eq!(count_kpop_entries(text), 2);
        assert_eq!(count_mbc2_entries(text), 1);
        assert_eq!(hypotheses_emitted(text), 3);
    }

    #[test]
    fn counts_steps_with_ascii_hyphen_separator() {
        let text = "## Step 1 - KPOP x\n## Step 2 - MBC2 y\n## Step 3 - KPOP z\n";
        assert_eq!(count_kpop_entries(text), 2);
        assert_eq!(count_mbc2_entries(text), 1);
        assert_eq!(hypotheses_emitted(text), 3);
    }

    #[test]
    fn counts_steps_with_en_dash_separator() {
        let text = "## Step 1 \u{2013} KPOP x\n## Step 2 \u{2013} MBC2 y\n";
        assert_eq!(count_kpop_entries(text), 1);
        assert_eq!(count_mbc2_entries(text), 1);
        assert_eq!(hypotheses_emitted(text), 2);
    }

    #[test]
    fn counts_steps_with_mixed_dash_styles() {
        let text = "## Step 1 — KPOP a\n## Step 2 - KPOP b\n## Step 3 \u{2013} MBC2 c\n";
        assert_eq!(count_kpop_entries(text), 2);
        assert_eq!(count_mbc2_entries(text), 1);
        assert_eq!(hypotheses_emitted(text), 3);
    }

    #[test]
    fn rejects_step_without_recognized_separator() {
        let text = "## Step 1 KPOP no-sep\n## Step 2: KPOP colon\n";
        assert_eq!(count_kpop_entries(text), 0);
        assert_eq!(hypotheses_emitted(text), 0);
    }

    #[test]
    fn success_marker_detected() {
        assert!(!agent_declared_success("no marker"));
        assert!(agent_declared_success("## KPOP_SOLVED\ndone"));
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
        let _ = stringify!(super::poisson_large_mean_normal_approx);
        let _ = stringify!(super::step_kind);
    }
}
