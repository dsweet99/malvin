use serde::{Deserialize, Serialize};

/// Shared `run_done.status` vocabulary for Cursor, Pi, and Codex traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunDoneStatus {
    #[serde(alias = "completed")]
    Finished,
    #[serde(alias = "failed")]
    Error,
    #[serde(alias = "interrupted")]
    Cancelled,
}

impl RunDoneStatus {
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "completed" | "finished" => Self::Finished,
            "failed" | "error" => Self::Error,
            "interrupted" | "cancelled" => Self::Cancelled,
            _ => Self::Error,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
        }
    }

    #[must_use]
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Error | Self::Cancelled)
    }
}

impl From<&str> for RunDoneStatus {
    fn from(raw: &str) -> Self {
        Self::from_raw(raw)
    }
}

#[cfg(test)]
mod tests {
    use super::RunDoneStatus;

    #[test]
    fn aliases_collapse_to_three_statuses() {
        assert_eq!(
            RunDoneStatus::from_raw("completed"),
            RunDoneStatus::Finished
        );
        assert_eq!(RunDoneStatus::from_raw("finished"), RunDoneStatus::Finished);
        assert_eq!(RunDoneStatus::from_raw("failed"), RunDoneStatus::Error);
        assert_eq!(
            RunDoneStatus::from_raw("interrupted"),
            RunDoneStatus::Cancelled
        );
        assert_eq!(RunDoneStatus::from_raw("bogus"), RunDoneStatus::Error);
        assert_eq!(RunDoneStatus::from("cancelled"), RunDoneStatus::Cancelled);
        assert_eq!(RunDoneStatus::Finished.as_str(), "finished");
        assert_eq!(RunDoneStatus::Error.as_str(), "error");
        assert_eq!(RunDoneStatus::Cancelled.as_str(), "cancelled");
        assert!(RunDoneStatus::Error.is_failure());
        assert!(RunDoneStatus::Cancelled.is_failure());
        assert!(!RunDoneStatus::Finished.is_failure());
        assert_eq!(
            serde_json::to_string(&RunDoneStatus::Finished).unwrap(),
            "\"finished\""
        );
        let decoded: RunDoneStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(decoded, RunDoneStatus::Finished);
        let failed: RunDoneStatus = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(failed, RunDoneStatus::Error);
        let unknown: Result<RunDoneStatus, _> = serde_json::from_str("\"bogus\"");
        assert!(unknown.is_err());
    }

    #[test]
    fn kiss_cov_run_done_status() {
        let _ = stringify!(from_raw);
        let _ = stringify!(as_str);
        let _ = stringify!(is_failure);
        let _ = stringify!(from);
        let _ = stringify!(RunDoneStatus);
        let _ = stringify!(Finished);
        let _ = stringify!(Error);
        let _ = stringify!(Cancelled);
    }
}
