//! Post-prompt plan file-shape validation for `malvin plan`.

use super::PlanFileError;

pub(crate) const SECTION_RESTATEMENT: &str = "## Restatement";
pub(crate) const SECTION_CRITIQUE: &str = "## Critique";
pub(crate) const SECTION_OPEN_QUESTIONS: &str = "## Open questions";
pub(crate) const SECTION_DECISIONS: &str = "## DECISIONS";

fn line_is_section_heading(line: &str, heading: &str) -> bool {
    line.trim() == heading
}

fn section_present(content: &str, section: &str) -> bool {
    content
        .lines()
        .any(|line| line_is_section_heading(line, section))
}

pub fn validate_post_1a(content: &str) -> Result<(), PlanFileError> {
    if !section_present(content, SECTION_RESTATEMENT) {
        return Err(PlanFileError::MissingSection(SECTION_RESTATEMENT));
    }
    Ok(())
}

pub fn validate_post_1b(content: &str) -> Result<(), PlanFileError> {
    validate_post_1a(content)?;
    if !section_present(content, SECTION_CRITIQUE) {
        return Err(PlanFileError::MissingSection(SECTION_CRITIQUE));
    }
    if !section_present(content, SECTION_OPEN_QUESTIONS) {
        return Err(PlanFileError::MissingSection(SECTION_OPEN_QUESTIONS));
    }
    Ok(())
}

pub fn validate_post_2(content: &str) -> Result<(), PlanFileError> {
    validate_post_1b(content)?;
    if !section_present(content, SECTION_DECISIONS) {
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
    fn line_is_section_heading_matches_trimmed_line() {
        assert!(line_is_section_heading("  ## Critique  ", SECTION_CRITIQUE));
        assert!(!line_is_section_heading("See ## Critique below.", SECTION_CRITIQUE));
    }

    #[test]
    fn section_present_rejects_heading_in_prose() {
        let content = format!(
            "{SECTION_RESTATEMENT}\nSee {SECTION_CRITIQUE} below.\n\n{SECTION_OPEN_QUESTIONS}\n1. q\n"
        );
        assert!(!section_present(&content, SECTION_CRITIQUE));
    }
}
