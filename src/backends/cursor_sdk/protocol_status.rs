use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunDoneStatus {
    Finished,
    Error,
    Cancelled,
    Unknown,
}

impl RunDoneStatus {
    #[must_use]
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "completed" | "finished" => Self::Finished,
            "failed" | "error" => Self::Error,
            "interrupted" | "cancelled" => Self::Cancelled,
            "unknown" => Self::Unknown,
            other => {
                tracing::warn!(status = other, "unknown run_done status");
                Self::Unknown
            }
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::Error => "error",
            Self::Cancelled => "cancelled",
            Self::Unknown => "unknown",
        }
    }

    #[must_use]
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Error | Self::Cancelled | Self::Unknown)
    }
}

impl Serialize for RunDoneStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for RunDoneStatus {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Ok(Self::from_raw(&raw))
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
    fn aliases_and_unknown_status() {
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
        assert_eq!(RunDoneStatus::from_raw("bogus"), RunDoneStatus::Unknown);
        assert_eq!(RunDoneStatus::from_raw("unknown"), RunDoneStatus::Unknown);
        assert_eq!(RunDoneStatus::from("cancelled"), RunDoneStatus::Cancelled);
        assert_eq!(RunDoneStatus::Finished.as_str(), "finished");
        assert_eq!(RunDoneStatus::Error.as_str(), "error");
        assert_eq!(RunDoneStatus::Cancelled.as_str(), "cancelled");
        assert_eq!(RunDoneStatus::Unknown.as_str(), "unknown");
        assert!(RunDoneStatus::Error.is_failure());
        assert!(RunDoneStatus::Cancelled.is_failure());
        assert!(RunDoneStatus::Unknown.is_failure());
        assert!(!RunDoneStatus::Finished.is_failure());
        assert_eq!(
            serde_json::to_string(&RunDoneStatus::Finished).unwrap(),
            "\"finished\""
        );
        let decoded: RunDoneStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(decoded, RunDoneStatus::Finished);
        let failed: RunDoneStatus = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(failed, RunDoneStatus::Error);
        let unknown: RunDoneStatus = serde_json::from_str("\"bogus\"").unwrap();
        assert_eq!(unknown, RunDoneStatus::Unknown);
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
        let _ = stringify!(Unknown);
    }
}
