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
    poisson_normal_draw_clamp(raw)
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn poisson_normal_draw_clamp(raw: f64) -> usize {
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

/// Reads the experiment log at `path` into a string.
///
/// # Errors
///
/// Returns `Err` when the file cannot be read.
pub fn read_exp_log_text(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read exp log {}: {e}", path.display()))
}

fn is_kpop_step_label(tail: &str) -> bool {
    if tail.len() < 4 || !tail[..4].eq_ignore_ascii_case("kpop") {
        return false;
    }
    tail.len() == 4 || !tail.as_bytes()[4].is_ascii_alphanumeric()
}

fn step_kind(line: &str) -> Option<&'static str> {
    let t = line.trim_start();
    let rest = t.strip_prefix("## Step ")?;
    let tail = [" — ", " – ", " - "]
        .iter()
        .find_map(|sep| rest.split_once(sep).map(|(_, t)| t))?;
    let tail = tail.trim_start();
    if is_kpop_step_label(tail) {
        return Some("KPop");
    }
    if tail.starts_with("MBC2") {
        return Some("MBC2");
    }
    None
}

#[must_use]
pub fn count_kpop_entries(text: &str) -> usize {
    text.lines()
        .filter(|line| step_kind(line) == Some("KPop"))
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

fn is_kpop_solved_marker_line(line: &str) -> bool {
    let t = line.trim_start();
    let Some(rest) = t.strip_prefix("## KPOP_SOLVED") else {
        return false;
    };
    rest.trim().is_empty()
}

#[must_use]
pub fn count_kpop_solved_markers(text: &str) -> usize {
    text.lines().filter(|line| is_kpop_solved_marker_line(line)).count()
}

#[must_use]
pub fn agent_declared_success(text: &str) -> bool {
    count_kpop_solved_markers(text) > 0
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{
        is_kpop_step_label, poisson_large_mean_normal_approx, poisson_normal_draw_clamp, step_kind,
    };

    #[test]
    fn step_kind_classifies_kpop_mbc2_and_rejects_kpopulation() {
        assert_eq!(step_kind("## Step 1 — KPop x"), Some("KPop"));
        assert_eq!(step_kind("## Step 2 — MBC2 y"), Some("MBC2"));
        assert_eq!(step_kind("## Step 3 — kpopulation x"), None);
    }

    #[test]
    fn is_kpop_step_label_accepts_kpop_prefix_only() {
        assert!(is_kpop_step_label("KPop"));
        assert!(is_kpop_step_label("kpop"));
        assert!(!is_kpop_step_label("kpopulation"));
        assert!(!is_kpop_step_label("foo"));
    }

    #[test]
    fn poisson_normal_draw_clamp_zero_and_max() {
        assert_eq!(poisson_normal_draw_clamp(-1.0), 0);
        assert_eq!(poisson_normal_draw_clamp(0.0), 0);
        assert_eq!(poisson_normal_draw_clamp(1.0e20), usize::MAX);
    }

    #[test]
    fn poisson_large_mean_normal_approx_returns_draw_near_mean() {
        let mut rng = StdRng::seed_from_u64(99);
        let n = poisson_large_mean_normal_approx(&mut rng, 5000.0);
        assert!(n > 1000 && n < 10_000);
    }
}

