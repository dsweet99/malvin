//! Shared channel routing helpers for agent backends.

use super::ObservabilityChannel;

/// Audit channel marker shared by agent emitters.
#[allow(dead_code)]
pub(crate) const AUDIT_CHANNEL: ObservabilityChannel = ObservabilityChannel::Audit;
/// Narrative channel marker shared by agent emitters.
#[allow(dead_code)]
pub(crate) const NARRATIVE_CHANNEL: ObservabilityChannel = ObservabilityChannel::Narrative;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_constants_are_distinct() {
        assert_ne!(AUDIT_CHANNEL, NARRATIVE_CHANNEL);
    }
}
