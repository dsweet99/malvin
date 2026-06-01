//! Plan run metadata persisted under `.malvin/logs/<run>/`.

use std::path::Path;

use super::{PlanFileError, PLAN_METADATA_FILE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRunMetadata {
    pub user_span_end: usize,
    pub user_span_sha256: Option<String>,
}

impl PlanRunMetadata {
    fn to_json(&self) -> String {
        self.user_span_sha256.as_ref().map_or_else(
            || format!("{{\n  \"user_span_end\": {}\n}}\n", self.user_span_end),
            |hash| format!(
                "{{\n  \"user_span_end\": {},\n  \"user_span_sha256\": \"{hash}\"\n}}\n",
                self.user_span_end
            ),
        )
    }

    fn from_json(text: &str) -> Result<Self, PlanFileError> {
        let value: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| PlanFileError::Io(format!("plan metadata parse: {e}")))?;
        let user_span_end = value
            .get("user_span_end")
            .and_then(serde_json::Value::as_u64)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| PlanFileError::Io("plan metadata missing user_span_end".to_string()))?;
        let user_span_sha256 = value
            .get("user_span_sha256")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        Ok(Self {
            user_span_end,
            user_span_sha256,
        })
    }
}

pub fn write_plan_metadata(run_dir: &Path, metadata: &PlanRunMetadata) -> Result<(), PlanFileError> {
    let path = run_dir.join(PLAN_METADATA_FILE);
    std::fs::write(path, metadata.to_json())?;
    Ok(())
}

pub fn read_plan_metadata(run_dir: &Path) -> Result<Option<PlanRunMetadata>, PlanFileError> {
    let path = run_dir.join(PLAN_METADATA_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path)?;
    PlanRunMetadata::from_json(&text).map(Some)
}
