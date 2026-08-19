use crate::malvin_constants::{STDOUT_LOG, TRACE_JSONL};
pub use crate::output::{WHO_B, WHO_H, WHO_M, WHO_O, WHO_T, WHO_U};

pub(crate) mod emit;
#[allow(unused_imports)]
pub(crate) use emit::{AUDIT_CHANNEL, NARRATIVE_CHANNEL};

pub const RUN_NARRATIVE_LOG: &str = STDOUT_LOG;
pub const RUN_AUDIT_LOG: &str = TRACE_JSONL;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityChannel {
    Narrative,
    Audit,
}

pub use crate::acp_trace_impersonation::SyntheticAcpSessionUpdate as AuditEventKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NarrativeWhoTag {
    Agent,
    Tool,
    User,
    Thought,
    Heartbeat,
    Ops,
}

impl NarrativeWhoTag {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Agent => WHO_M,
            Self::Tool => WHO_T,
            Self::User => WHO_U,
            Self::Thought => WHO_B,
            Self::Heartbeat => WHO_H,
            Self::Ops => WHO_O,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observability_channel_variants_stable() {
        assert_ne!(ObservabilityChannel::Narrative, ObservabilityChannel::Audit);
    }

    #[test]
    fn run_log_aliases_match_malvin_constants() {
        assert_eq!(RUN_NARRATIVE_LOG, "stdout.log");
        assert_eq!(RUN_AUDIT_LOG, "trace.jsonl");
    }

    #[test]
    fn narrative_who_tag_covers_all_roles() {
        for tag in [
            NarrativeWhoTag::Agent,
            NarrativeWhoTag::Tool,
            NarrativeWhoTag::User,
            NarrativeWhoTag::Thought,
            NarrativeWhoTag::Heartbeat,
            NarrativeWhoTag::Ops,
        ] {
            assert_eq!(tag.as_str().len(), 1);
        }
    }

    #[test]
    fn synthetic_acp_session_update_round_trips_audit_event_kind_alias() {
        use crate::acp_trace_impersonation::SyntheticAcpSessionUpdate;
        use crate::observability::AuditEventKind;
        for variant in SyntheticAcpSessionUpdate::all() {
            let alias: AuditEventKind = *variant;
            assert_eq!(format!("{alias:?}"), format!("{variant:?}"));
        }
    }

    #[test]
    fn audit_event_kind_variants_are_distinct() {
        use std::collections::HashSet;
        let kinds = [
            AuditEventKind::AgentMessageChunk,
            AuditEventKind::AgentThoughtChunk,
            AuditEventKind::ToolCall,
            AuditEventKind::ToolCallUpdate,
            AuditEventKind::OutRaw,
        ];
        let set: HashSet<_> = kinds.into_iter().collect();
        assert_eq!(set.len(), kinds.len());
        for kind in kinds {
            assert!(!format!("{kind:?}").is_empty());
        }
    }

    #[test]
    fn module_doc_mentions_dual_contract() {
        let doc = include_str!("mod.rs");
        assert!(
            doc.contains("Dual-contract"),
            "module doc must name dual-contract observability"
        );
    }
}
