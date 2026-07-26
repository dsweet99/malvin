//! Parse agent-written `review_requirements.json` for the default route.

use std::path::Path;

pub const MAX_REVIEW_REQUIREMENT_GROUPS: usize = 5;
pub const MAX_REQUIREMENTS_PER_GROUP: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRequirements {
    pub groups: Vec<ReviewRequirementGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRequirementGroup {
    pub title: Option<String>,
    pub requirements: Vec<String>,
}

impl ReviewRequirementGroup {
    #[must_use]
    pub fn title_trimmed(&self) -> String {
        self.title.as_deref().map_or("", str::trim)
            .to_string()
    }

    #[must_use]
    pub fn requirements_bullet_list(&self) -> String {
        self.requirements
            .iter()
            .map(|r| format!("- {r}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// # Errors
/// Returns an error when the file is missing, not valid JSON, or violates group/requirement caps.
pub(crate) fn load_review_requirements(path: &Path) -> Result<ReviewRequirements, String> {
    let raw = std::fs::read_to_string(path).map_err(|e| {
        format!(
            "review_requirements: missing or unreadable file at {}: {e}",
            path.display()
        )
    })?;
    parse_review_requirements_json(&raw)
}

/// # Errors
/// Returns an error when JSON is malformed or violates group/requirement caps.
pub(crate) fn parse_review_requirements_json(raw: &str) -> Result<ReviewRequirements, String> {
    let value: serde_json::Value = serde_json::from_str(raw).map_err(|e| {
        format!("review_requirements: malformed JSON: {e}")
    })?;
    let groups_val = value
        .get("groups")
        .ok_or_else(|| "review_requirements: missing top-level \"groups\" array".to_string())?;
    let groups_arr = groups_val
        .as_array()
        .ok_or_else(|| "review_requirements: \"groups\" must be an array".to_string())?;
    let mut groups = Vec::with_capacity(groups_arr.len());
    for (i, group_val) in groups_arr.iter().enumerate() {
        groups.push(parse_group(i, group_val)?);
    }
    let parsed = ReviewRequirements { groups };
    validate_review_requirements(&parsed)?;
    Ok(parsed)
}

fn parse_group(index: usize, value: &serde_json::Value) -> Result<ReviewRequirementGroup, String> {
    let obj = value.as_object().ok_or_else(|| {
        format!("review_requirements: group {index} must be an object")
    })?;
    let title = match obj.get("title") {
        None => None,
        Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(_) => {
            return Err(format!(
                "review_requirements: group {index} \"title\" must be a string when present"
            ));
        }
    };
    let reqs_val = obj.get("requirements").ok_or_else(|| {
        format!("review_requirements: group {index} missing \"requirements\" array")
    })?;
    let reqs_arr = reqs_val.as_array().ok_or_else(|| {
        format!("review_requirements: group {index} \"requirements\" must be an array")
    })?;
    let mut requirements = Vec::with_capacity(reqs_arr.len());
    for (j, req) in reqs_arr.iter().enumerate() {
        let s = req.as_str().ok_or_else(|| {
            format!("review_requirements: group {index} requirement {j} must be a string")
        })?;
        requirements.push(s.to_string());
    }
    Ok(ReviewRequirementGroup { title, requirements })
}

fn validate_review_requirements(parsed: &ReviewRequirements) -> Result<(), String> {
    if parsed.groups.len() > MAX_REVIEW_REQUIREMENT_GROUPS {
        return Err(format!(
            "review_requirements: too many groups ({}); max is {MAX_REVIEW_REQUIREMENT_GROUPS}",
            parsed.groups.len()
        ));
    }
    for (i, group) in parsed.groups.iter().enumerate() {
        let n = group.requirements.len();
        if n == 0 || n > MAX_REQUIREMENTS_PER_GROUP {
            return Err(format!(
                "review_requirements: group {i} has {n} requirements; each group must have 1..={MAX_REQUIREMENTS_PER_GROUP}"
            ));
        }
        for (j, req) in group.requirements.iter().enumerate() {
            if req.trim().is_empty() {
                return Err(format!(
                    "review_requirements: group {i} requirement {j} is empty after trim"
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod review_requirements_parse_tests {
    use super::{
        load_review_requirements, parse_review_requirements_json, ReviewRequirementGroup,
        ReviewRequirements, MAX_REQUIREMENTS_PER_GROUP, MAX_REVIEW_REQUIREMENT_GROUPS,
    };

    #[test]
    fn kiss_cov_review_requirements_structs() {
        let group = ReviewRequirementGroup {
            title: Some(" t ".to_string()),
            requirements: vec!["a".to_string()],
        };
        let ReviewRequirementGroup {
            title,
            requirements,
        } = group.clone();
        assert_eq!(title.as_deref(), Some(" t "));
        assert_eq!(requirements, ["a"]);
        assert_eq!(group.title_trimmed(), "t");
        assert_eq!(group.requirements_bullet_list(), "- a");
        let reqs = ReviewRequirements {
            groups: vec![group],
        };
        let ReviewRequirements { groups } = reqs;
        assert_eq!(groups.len(), 1);
    }

    #[test]
    fn accepts_zero_one_and_five_groups() {
        let zero = parse_review_requirements_json(r#"{"groups":[]}"#).expect("zero");
        assert!(zero.groups.is_empty());

        let one = parse_review_requirements_json(
            r#"{"groups":[{"title":"A","requirements":["do thing"]}]}"#,
        )
        .expect("one");
        assert_eq!(one.groups.len(), 1);
        assert_eq!(one.groups[0].title_trimmed(), "A");
        assert_eq!(one.groups[0].requirements_bullet_list(), "- do thing");

        let five_raw = format!(
            r#"{{"groups":[{}]}}"#,
            (0..MAX_REVIEW_REQUIREMENT_GROUPS)
                .map(|i| format!(r#"{{"title":"g{i}","requirements":["req-1"]}}"#))
                .collect::<Vec<_>>()
                .join(",")
        );
        let parsed = parse_review_requirements_json(&five_raw).expect("five");
        assert_eq!(parsed.groups.len(), MAX_REVIEW_REQUIREMENT_GROUPS);
    }

    #[test]
    fn rejects_six_groups_and_six_requirements() {
        let six_groups = format!(
            r#"{{"groups":[{}]}}"#,
            (0..=MAX_REVIEW_REQUIREMENT_GROUPS)
                .map(|i| format!(r#"{{"requirements":["r{i}"]}}"#))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(parse_review_requirements_json(&six_groups).is_err());

        let six_reqs = format!(
            r#"{{"groups":[{{"requirements":[{}]}}]}}"#,
            (0..=MAX_REQUIREMENTS_PER_GROUP)
                .map(|i| format!(r#""r{i}""#))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(parse_review_requirements_json(&six_reqs).is_err());
    }

    #[test]
    fn rejects_missing_malformed_and_empty_requirement() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nope.json");
        assert!(load_review_requirements(&missing).is_err());
        assert!(parse_review_requirements_json("not json").is_err());
        assert!(parse_review_requirements_json(r#"{"groups":[{"requirements":[""]}]}"#).is_err());
        assert!(parse_review_requirements_json(r#"{"groups":[{"requirements":[]}]}"#).is_err());
    }
}
