#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThinkingLevel {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
    Ultra,
}

impl ThinkingLevel {
    pub(crate) const ALL: [Self; 8] = [
        Self::Off,
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Xhigh,
        Self::Max,
        Self::Ultra,
    ];

    pub(crate) const PI: [Self; 7] = [
        Self::Off,
        Self::Minimal,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::Xhigh,
        Self::Max,
    ];

    pub(crate) const CLAUDE_EFFORT: [Self; 5] =
        [Self::Low, Self::Medium, Self::High, Self::Xhigh, Self::Max];

    #[must_use]
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
            Self::Ultra => "ultra",
        }
    }

    #[must_use]
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.name() == value)
    }

    #[must_use]
    pub(crate) const fn pi_needs_explicit_level_map(self) -> bool {
        matches!(self, Self::Xhigh | Self::Max)
    }

    #[must_use]
    pub(crate) const fn pi_wire(self) -> Self {
        match self {
            Self::Ultra => Self::Max,
            other => other,
        }
    }

    #[must_use]
    pub(crate) const fn codex_wire(self) -> Self {
        match self {
            Self::Off | Self::Minimal => Self::Low,
            other => other,
        }
    }
}

#[must_use]
pub(crate) fn level_names(levels: &[ThinkingLevel]) -> Vec<&'static str> {
    levels.iter().map(|level| level.name()).collect()
}

#[must_use]
pub(crate) fn joined_level_names(levels: &[ThinkingLevel]) -> String {
    level_names(levels).join("|")
}

#[must_use]
pub(crate) fn pi_wire_thinking(value: &str) -> &str {
    ThinkingLevel::parse(value).map_or(value, |level| level.pi_wire().name())
}

#[must_use]
pub(crate) fn codex_wire_thinking(value: &str) -> &str {
    ThinkingLevel::parse(value).map_or(value, |level| level.codex_wire().name())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip_through_parse() {
        for level in ThinkingLevel::ALL {
            assert_eq!(ThinkingLevel::parse(level.name()), Some(level));
        }
        assert_eq!(ThinkingLevel::parse("ultra_high"), None);
    }

    #[test]
    fn backend_subsets_come_from_the_shared_vocabulary() {
        assert_eq!(
            joined_level_names(&ThinkingLevel::ALL),
            "off|minimal|low|medium|high|xhigh|max|ultra"
        );
        assert_eq!(
            joined_level_names(&ThinkingLevel::PI),
            "off|minimal|low|medium|high|xhigh|max"
        );
        assert_eq!(
            joined_level_names(&ThinkingLevel::CLAUDE_EFFORT),
            "low|medium|high|xhigh|max"
        );
    }

    #[test]
    fn every_level_maps_onto_a_level_the_backend_accepts() {
        for level in ThinkingLevel::ALL {
            assert!(ThinkingLevel::PI.contains(&level.pi_wire()), "{level:?}");
        }
        assert_eq!(pi_wire_thinking("ultra"), "max");
        assert_eq!(pi_wire_thinking("high"), "high");
        assert_eq!(codex_wire_thinking("off"), "low");
        assert_eq!(codex_wire_thinking("minimal"), "low");
        assert_eq!(codex_wire_thinking("ultra"), "ultra");
        assert_eq!(codex_wire_thinking("custom"), "custom");
    }
}
