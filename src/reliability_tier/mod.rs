#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReliabilityTier {
    Tenacious,
    Conservative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReliabilityTierFlags {
    pub tenacious: bool,
    pub no_tenacious: bool,
}

impl ReliabilityTier {
    #[must_use]
    pub const fn resolve(flags: ReliabilityTierFlags) -> Self {
        if flags.tenacious && !flags.no_tenacious {
            Self::Tenacious
        } else {
            Self::Conservative
        }
    }
}

#[cfg(test)]
#[path = "reliability_tier_tests.rs"]
mod reliability_tier_tests;
