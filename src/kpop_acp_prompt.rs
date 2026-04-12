//! Standalone KPOP ACP user-message selection (optional MBC2 creative branch).

use rand::Rng;
use rand::distributions::{Distribution, Uniform};

const MBC2_SUFFIX: &str = "\n\nGenerate one MBC2 hypothesis.";

/// First this many `session/prompt` calls in a KPOP run skip the creative branch.
pub const CREATIVE_MIN_INTERACTION: u32 = 3;

/// How many `session/prompt` calls standalone `malvin kpop` performs when `--p-creative` > 0 so
/// interaction index [`CREATIVE_MIN_INTERACTION`] exists and the MBC2 branch can apply (see `acp::run_kpop_flow_once`).
pub const KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE: u32 = CREATIVE_MIN_INTERACTION + 1;

const _: () = assert!(KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE > CREATIVE_MIN_INTERACTION);

/// `true` when standalone KPOP should load `mbc2.md` and may send extra pad/roll prompts.
///
/// Matches [`kpop_acp_user_prompt`] and [`kpop_standalone_outbound_prompt_count`]: non-finite or
/// non-positive values disable the creative path (unlike raw `p_creative > 0.0`, which is true for `+∞`).
#[must_use]
pub fn kpop_creative_enabled(p_creative: f64) -> bool {
    p_creative.is_finite() && p_creative > 0.0
}

/// Outbound `session/prompt` count for standalone `malvin kpop` (main + optional learn + optional creative rounds).
#[must_use]
pub fn kpop_standalone_outbound_prompt_count(p_creative: f64, has_learn: bool) -> u32 {
    if !kpop_creative_enabled(p_creative) {
        return u32::from(has_learn).saturating_add(1);
    }
    KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE
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
    if pick.interaction_index < CREATIVE_MIN_INTERACTION || p <= 0.0 {
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
    fn first_three_interactions_never_use_mbc2_even_at_probability_one() {
        let mut rng = StdRng::seed_from_u64(1);
        for idx in 0..CREATIVE_MIN_INTERACTION {
            let pick = KpopAcpPromptPick {
                interaction_index: idx,
                p_creative: 1.0,
                default_prompt: "DEFAULT",
                mbc2_body: "MBC2",
            };
            let out = kpop_acp_user_prompt(&pick, &mut rng);
            assert_eq!(out, "DEFAULT");
        }
    }

    #[test]
    fn fourth_interaction_can_switch_when_probability_one() {
        let mut rng = StdRng::seed_from_u64(1);
        let pick = KpopAcpPromptPick {
            interaction_index: CREATIVE_MIN_INTERACTION,
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
    fn kpop_acp_session_must_send_enough_prompts_for_p_creative_to_apply() {
        use super::{
            KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE, kpop_standalone_outbound_prompt_count,
        };

        assert_eq!(
            kpop_standalone_outbound_prompt_count(0.1, false),
            KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE
        );
        assert_eq!(
            kpop_standalone_outbound_prompt_count(0.1, true),
            KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE
        );
        assert_eq!(kpop_standalone_outbound_prompt_count(0.0, false), 1);
        assert_eq!(kpop_standalone_outbound_prompt_count(0.0, true), 2);
    }

    #[test]
    fn creative_enabled_false_for_positive_infinity() {
        use super::kpop_creative_enabled;

        assert!(!kpop_creative_enabled(f64::INFINITY));
    }

    #[test]
    fn creative_gate_matches_extra_prompt_count() {
        use super::{kpop_creative_enabled, kpop_standalone_outbound_prompt_count};

        for p in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            0.0,
            -0.0,
            -1.0,
            0.1,
            1.0,
        ] {
            let base = u32::from(false).saturating_add(1);
            let extra = kpop_standalone_outbound_prompt_count(p, false) > base;
            assert_eq!(kpop_creative_enabled(p), extra, "p={p:?}");
        }
    }
}
