//! Standalone KPOP ACP user-message selection (optional MBC2 creative branch).

use rand::Rng;
use rand::distributions::{Distribution, Uniform};

const MBC2_SUFFIX: &str = "\n\nGenerate one MBC2 hypothesis.";

/// First this many outbound `session/prompt` indices skip the MBC2 branch (even when `p_creative`
/// is 1.0). **0** disables that skip so only `p_creative` and the random roll apply—standalone KPOP
/// uses real prompts (main, optional `learn`) with no synthetic continuation rounds.
pub const CREATIVE_MIN_INTERACTION: u32 = 0;

/// When [`CREATIVE_MIN_INTERACTION`] is **0**, the `else` branch is unreachable; it remains so the
/// threshold can be raised without rewriting the gate.
#[allow(clippy::absurd_extreme_comparisons)]
const fn skip_mbc2_for_interaction_index(interaction_index: u32) -> bool {
    if CREATIVE_MIN_INTERACTION == 0 {
        false
    } else {
        interaction_index < CREATIVE_MIN_INTERACTION
    }
}

/// `true` when standalone KPOP should load `mbc2.md` and may apply the MBC2 suffix on outbound
/// prompts (via [`kpop_acp_user_prompt`]).
///
/// Standalone KPOP does not add extra `session/prompt` rounds for creative mode: non-finite or
/// non-positive `p_creative` values disable the creative path (unlike raw `p_creative > 0.0`, which
/// is true for `+∞`).
#[must_use]
pub fn kpop_creative_enabled(p_creative: f64) -> bool {
    p_creative.is_finite() && p_creative > 0.0
}

/// Inputs for [`kpop_acp_user_prompt`].
#[derive(Debug, Clone, Copy)]
pub struct KpopAcpPromptPick<'a> {
    pub interaction_index: u32,
    pub p_creative: f64,
    pub default_prompt: &'a str,
    pub mbc2_body: &'a str,
}

#[must_use]
pub fn kpop_acp_user_prompt(pick: &KpopAcpPromptPick<'_>, rng: &mut impl Rng) -> String {
    let p = if pick.p_creative.is_finite() {
        pick.p_creative.clamp(0.0, 1.0)
    } else {
        0.0
    };
    if skip_mbc2_for_interaction_index(pick.interaction_index) || p <= 0.0 {
        return pick.default_prompt.to_string();
    }
    let roll = Uniform::from(0.0..1.0).sample(rng);
    if roll < p {
        format!("{}{}", pick.mbc2_body.trim_end(), MBC2_SUFFIX)
    } else {
        pick.default_prompt.to_string()
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    use super::{CREATIVE_MIN_INTERACTION, KpopAcpPromptPick, kpop_acp_user_prompt};

    #[test]
    fn first_interaction_can_switch_when_probability_one() {
        assert_eq!(CREATIVE_MIN_INTERACTION, 0);
        let mut rng = StdRng::seed_from_u64(1);
        let pick = KpopAcpPromptPick {
            interaction_index: 0,
            p_creative: 1.0,
            default_prompt: "DEFAULT",
            mbc2_body: "MBC2",
        };
        let out = kpop_acp_user_prompt(&pick, &mut rng);
        assert!(out.starts_with("MBC2"));
        assert!(out.contains("Generate one MBC2 hypothesis."));
    }

    #[test]
    fn zero_probability_keeps_default_after_min_interactions() {
        let mut rng = StdRng::seed_from_u64(99);
        let pick = KpopAcpPromptPick {
            interaction_index: 10,
            p_creative: 0.0,
            default_prompt: "DEFAULT",
            mbc2_body: "MBC2",
        };
        let out = kpop_acp_user_prompt(&pick, &mut rng);
        assert_eq!(out, "DEFAULT");
    }

    #[test]
    fn creative_enabled_false_for_positive_infinity() {
        use super::kpop_creative_enabled;

        assert!(!kpop_creative_enabled(f64::INFINITY));
    }

    #[test]
    fn skip_mbc2_for_interaction_index_always_false_when_min_is_zero() {
        use super::skip_mbc2_for_interaction_index;

        assert_eq!(CREATIVE_MIN_INTERACTION, 0, "test assumes min is 0");
        assert!(!skip_mbc2_for_interaction_index(0));
        assert!(!skip_mbc2_for_interaction_index(1));
        assert!(!skip_mbc2_for_interaction_index(100));
    }
}
