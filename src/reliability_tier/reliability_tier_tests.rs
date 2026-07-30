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
fn reliability_tier_variants_exist() {
    let _ = (ReliabilityTier::Tenacious, ReliabilityTier::Conservative);
}
