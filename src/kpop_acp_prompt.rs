//! Standalone KPOP: [`kpop_creative_enabled`] gates pre-generated schedule MBC2 slots.

#[allow(dead_code)]
pub const CREATIVE_MIN_INTERACTION: u32 = 0;

/// `true` when `p_creative` may place [`crate::kpop_schedule::KpopScheduleStep::Mbc2ThenFalsify`]
/// steps in the pre-generated schedule.
///
/// Non-finite or non-positive `p_creative` values disable MBC2 scheduling (unlike raw `p_creative >
/// 0.0`, which is true for `+∞`).
#[must_use]
pub fn kpop_creative_enabled(p_creative: f64) -> bool {
    p_creative.is_finite() && p_creative > 0.0
}

#[cfg(test)]
mod tests {
    use super::{CREATIVE_MIN_INTERACTION, kpop_creative_enabled};

    #[test]
    fn creative_enabled_false_for_positive_infinity() {
        assert!(!kpop_creative_enabled(f64::INFINITY));
    }

    #[test]
    fn creative_min_interaction_stays_zero_for_cli_parity() {
        assert_eq!(CREATIVE_MIN_INTERACTION, 0);
    }
}
