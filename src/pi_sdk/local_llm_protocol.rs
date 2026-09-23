use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "snake_case")]
pub(crate) enum ManagerRequest {
    Ensure { provider: String, model: String },
    Hold,
    Release,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ManagerResponse {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

impl ManagerResponse {
    pub(crate) const fn ok_ping(pid: u32) -> Self {
        Self {
            ok: true,
            error: None,
            pid: Some(pid),
        }
    }

    pub(crate) const fn ok_ensure() -> Self {
        Self {
            ok: true,
            error: None,
            pid: None,
        }
    }

    pub(crate) fn err(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(message.into()),
            pid: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_and_hold_round_trip_json() {
        let ensure = ManagerRequest::Ensure {
            provider: "ollama".into(),
            model: "toy".into(),
        };
        let text = serde_json::to_string(&ensure).expect("ser");
        assert_eq!(serde_json::from_str::<ManagerRequest>(&text).unwrap(), ensure);
        let hold = ManagerRequest::Hold;
        let text = serde_json::to_string(&hold).expect("ser");
        assert_eq!(serde_json::from_str::<ManagerRequest>(&text).unwrap(), hold);
    }
}
