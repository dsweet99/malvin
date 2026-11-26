use std::path::PathBuf;

const FACTS_FILE: &str = "facts.jsonl";

pub(crate) const FIELDS: &str = "fact_id, description, fact, symptoms";

pub(crate) fn facts_path() -> PathBuf {
    crate::malvin_user_home_root().join(FACTS_FILE)
}

#[derive(Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct FactRecord {
    pub fact_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fact: String,
    #[serde(default)]
    pub symptoms: Vec<String>,
}

impl FactRecord {
    pub(crate) fn from_line(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }
}

pub(crate) fn parse_facts(text: &str) -> impl Iterator<Item = FactRecord> + '_ {
    text.lines().filter_map(FactRecord::from_line)
}
