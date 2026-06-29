use crate::cli::{TENACIOUS_MAX_ACP_RETRIES, TENACIOUS_MAX_LOOPS};
use crate::config::DEFAULT_MAX_ACP_RETRIES;
use crate::malvin_config_file::DEFAULT_MAX_LOOPS;

use super::{ReliabilityTier, ReliabilityTierFlags};

#[test]
fn reliability_tier_resolve_logic() {
    assert_eq!(
        ReliabilityTier::resolve(ReliabilityTierFlags {
            tenacious: true,
            no_tenacious: false,
        }),
        ReliabilityTier::Tenacious
    );
    assert_eq!(
        ReliabilityTier::resolve(ReliabilityTierFlags {
            tenacious: true,
            no_tenacious: true,
        }),
        ReliabilityTier::Conservative
    );
    assert_eq!(
        ReliabilityTier::resolve(ReliabilityTierFlags {
            tenacious: false,
            no_tenacious: false,
        }),
        ReliabilityTier::Conservative
    );
    assert_eq!(
        ReliabilityTier::resolve(ReliabilityTierFlags {
            tenacious: false,
            no_tenacious: true,
        }),
        ReliabilityTier::Conservative
    );
}

#[test]
fn reliability_tier_tenacious_budget_constants() {
    let tier = ReliabilityTier::Tenacious;
    assert_eq!(tier.default_max_loops(), TENACIOUS_MAX_LOOPS);
    assert_eq!(tier.default_max_acp_retries(), TENACIOUS_MAX_ACP_RETRIES);
}

#[test]
fn reliability_tier_conservative_budget_constants() {
    let tier = ReliabilityTier::Conservative;
    assert_eq!(tier.default_max_loops(), DEFAULT_MAX_LOOPS);
    assert_eq!(tier.default_max_acp_retries(), DEFAULT_MAX_ACP_RETRIES);
}

#[test]
fn reliability_tier_variants_exist() {
    let _ = (ReliabilityTier::Tenacious, ReliabilityTier::Conservative);
}
