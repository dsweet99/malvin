//! Shared channel routing helpers for mini and ACP backends.

use super::ObservabilityChannel;

/// Audit channel marker shared by mini and ACP emitters.
pub(crate) const AUDIT_CHANNEL: ObservabilityChannel = ObservabilityChannel::Audit;
/// Narrative channel marker shared by mini and ACP emitters.
pub(crate) const NARRATIVE_CHANNEL: ObservabilityChannel = ObservabilityChannel::Narrative;

/// Returns true when narrative stdout should be suppressed.
#[must_use]
pub(crate) fn narrative_suppressed(no_tee: bool) -> bool {
    no_tee || crate::output::stdout_suppressed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_constants_are_distinct() {
        assert_ne!(AUDIT_CHANNEL, NARRATIVE_CHANNEL);
    }

    #[test]
    fn narrative_suppressed_when_no_tee() {
        assert!(narrative_suppressed(true));
    }
}
