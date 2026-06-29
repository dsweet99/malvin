//! Dual-contract observability for malvin runs.
//!
//! Malvin maintains two parallel output channels with different contracts:
//!
//! | Channel | Artifact | Contract |
//! |---|---|---|
//! | **Narrative** | live stdout + `stdout.log` | Lossy, human-oriented stream with who-tags |
//! | **Audit** | `trace.jsonl` | Machine-authoritative ACP-shaped JSONL |
//!
//! **Trust rule:** Consumers must know which channel to trust for which question.
//! - Tool exit codes, LLM usage, shrink/fork events → **audit** (`trace.jsonl`)
//! - Human skimming, vocabulary/ordering parity → **narrative** (`stdout.log`)
//! - Prompt bodies (full text) → audit `out` lines and/or `prompts.log`, not narrative by default
//!
//! Adjacent artifact: `prompts.log` holds outgoing prompt bodies; it is not part of the
//! two-channel model above.
//!
//! **Where:** narrative emission lives in [`crate::output`]; mini audit emission in
//! [`crate::agent_backend::mini::trace`] and [`crate::agent_backend::mini::acp_trace_shim`].

use crate::malvin_constants::{STDOUT_LOG, TRACE_JSONL};
pub use crate::output::{WHO_B, WHO_H, WHO_M, WHO_O, WHO_T, WHO_U};

pub(crate) mod emit;
pub(crate) use emit::{narrative_suppressed, AUDIT_CHANNEL, NARRATIVE_CHANNEL};

/// Run-directory log filenames for the two observability channels.
pub const RUN_NARRATIVE_LOG: &str = STDOUT_LOG;
pub const RUN_AUDIT_LOG: &str = TRACE_JSONL;

/// Which output channel an emission targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObservabilityChannel {
    /// Lossy human-oriented stream (`stdout` / `stdout.log`).
    Narrative,
    /// Machine-authoritative semantic record (`trace.jsonl`).
    Audit,
}

/// Known audit record kinds (ACP session updates and mini-only extensions).
pub use crate::acp_trace_impersonation::SyntheticAcpSessionUpdate as AuditEventKind;

/// Who-tag on narrative lines (`m|`, `t|`, …). See [`crate::output`] for formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NarrativeWhoTag {
    /// Normal agent output (`m|`).
    Agent,
    /// Tool call summaries (`t|`).
    Tool,
    /// User input / outgoing prompt bracket (`u|`).
    User,
    /// Thinking / thought chunks (`b|`).
    Thought,
    /// Heartbeats (`h|`).
    Heartbeat,
    /// General operational info (`o|`).
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

/// JSON field names on mini audit `session/update` records that must never appear in `stdout.log`.
const AUDIT_ONLY_SESSION_UPDATE_FIELDS: &[&str] = &[
    "miniTerminal",
    "miniHttpExchange",
    "miniPromptShrink",
    "miniPromptShrinkStalled",
    "miniRetryFork",
    "miniUsage",
];

/// Returns true when `field` is a mini audit-only extension key on `agent_message_chunk` records.
#[must_use]
pub fn is_audit_only_session_update(field: &str) -> bool {
    AUDIT_ONLY_SESSION_UPDATE_FIELDS.contains(&field)
}

/// All audit-only mini extension field names (for contract tests and future lint).
#[must_use]
pub const fn audit_only_session_update_fields() -> &'static [&'static str] {
    AUDIT_ONLY_SESSION_UPDATE_FIELDS
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
    fn is_audit_only_session_update_classifies_mini_extensions() {
        assert!(is_audit_only_session_update("miniTerminal"));
        assert!(is_audit_only_session_update("miniHttpExchange"));
        assert!(!is_audit_only_session_update("sessionUpdate"));
        assert!(!is_audit_only_session_update("content"));
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
            AuditEventKind::LlmUsage,
            AuditEventKind::MiniTerminal,
            AuditEventKind::MiniHttpExchange,
            AuditEventKind::MiniPromptShrink,
            AuditEventKind::MiniPromptShrinkStalled,
            AuditEventKind::MiniRetryFork,
        ];
        let set: HashSet<_> = kinds.into_iter().collect();
        assert_eq!(set.len(), kinds.len());
        for kind in kinds {
            assert!(!format!("{kind:?}").is_empty());
        }
    }

    #[test]
    fn audit_only_fields_list_is_complete() {
        assert!(audit_only_session_update_fields().contains(&"miniTerminal"));
        assert_eq!(audit_only_session_update_fields().len(), 6);
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
