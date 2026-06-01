//! Post-prompt plan file-shape validation for `malvin plan`.

use super::{detect_rerun_user_span_end, PlanFileError, BEGIN_MALVIN_MARKER};

pub(crate) const SECTION_RESTATEMENT: &str = "## Restatement";
pub(crate) const SECTION_CRITIQUE: &str = "## Critique";
pub(crate) const SECTION_OPEN_QUESTIONS: &str = "## Open questions";
pub(crate) const SECTION_DECISIONS: &str = "## DECISIONS";

pub fn record_user_span_end_after_1a(content: &str) -> Result<usize, PlanFileError> {
    detect_rerun_user_span_end(content)?
        .ok_or_else(|| PlanFileError::MissingSection("--- BEGIN_MALVIN after Prompt 1a"))
}

fn section_present_after_marker(content: &str, section: &str) -> bool {
    let Some(idx) = content.find(BEGIN_MALVIN_MARKER) else {
        return false;
    };
    content[idx..].contains(section)
}

pub fn validate_post_1a(content: &str) -> Result<(), PlanFileError> {
    record_user_span_end_after_1a(content)?;
    if !section_present_after_marker(content, SECTION_RESTATEMENT) {
        return Err(PlanFileError::MissingSection(SECTION_RESTATEMENT));
    }
    Ok(())
}

pub fn validate_post_1b(content: &str) -> Result<(), PlanFileError> {
    validate_post_1a(content)?;
    if !section_present_after_marker(content, SECTION_CRITIQUE) {
        return Err(PlanFileError::MissingSection(SECTION_CRITIQUE));
    }
    if !section_present_after_marker(content, SECTION_OPEN_QUESTIONS) {
        return Err(PlanFileError::MissingSection(SECTION_OPEN_QUESTIONS));
    }
    Ok(())
}

pub fn validate_post_2(content: &str) -> Result<(), PlanFileError> {
    validate_post_1b(content)?;
    if !section_present_after_marker(content, SECTION_DECISIONS) {
        return Err(PlanFileError::MissingSection(SECTION_DECISIONS));
    }
    Ok(())
}

pub fn extract_decisions_section(content: &str) -> Option<String> {
    let start = content.find(SECTION_DECISIONS)?;
    Some(content[start..].trim_end().to_string())
}

#[cfg(test)]
mod private_fn_coverage {
    use super::*;

    #[test]
    fn section_present_after_marker_without_begin_malvin() {
        assert!(!section_present_after_marker("## Restatement only", SECTION_RESTATEMENT));
    }

    #[test]
    fn section_present_after_marker_with_begin_malvin() {
        let content = format!("{BEGIN_MALVIN_MARKER}\n{SECTION_RESTATEMENT}\n");
        assert!(section_present_after_marker(&content, SECTION_RESTATEMENT));
    }
}
