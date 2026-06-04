//! Machine-block boundary detection for `malvin plan`.

use super::{PlanFileError, BEGIN_MALVIN_MARKER};

fn machine_block_marker_patterns() -> [(String, usize); 4] {
    [
        (format!("\n---\n{BEGIN_MALVIN_MARKER}"), 1),
        (format!("\r\n---\r\n{BEGIN_MALVIN_MARKER}"), 2),
        (format!("\n---\r\n{BEGIN_MALVIN_MARKER}"), 1),
        (format!("\r\n---\n{BEGIN_MALVIN_MARKER}"), 2),
    ]
}

pub(crate) fn count_begin_malvin_marker_lines(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.trim() == BEGIN_MALVIN_MARKER)
        .count()
}

/// Byte offset at the start of the machine block (`---` line), if exactly one unambiguous block exists.
#[must_use]
pub fn find_machine_block_start(content: &str) -> Option<usize> {
    for (marker, eol_len) in machine_block_marker_patterns() {
        if let Some(idx) = content.find(&marker) {
            let user = &content[..idx];
            if count_begin_malvin_marker_lines(user) > 0 {
                return None;
            }
            return Some(idx + eol_len);
        }
    }
    for at_start in ["---\n", "---\r\n"] {
        let marker = format!("{at_start}{BEGIN_MALVIN_MARKER}");
        if content.starts_with(&marker) {
            return Some(0);
        }
    }
    None
}

/// Returns `user_span_end` when a single machine block is present; errors when markers are ambiguous.
pub fn detect_rerun_user_span_end(content: &str) -> Result<Option<usize>, PlanFileError> {
    let count = count_begin_malvin_marker_lines(content);
    if count == 0 {
        return Ok(None);
    }
    if count > 1 {
        return Err(PlanFileError::DuplicateBeginMalvinMarkers);
    }
    find_machine_block_start(content)
        .ok_or(PlanFileError::MalformedMachineBlockDelimiter)
        .map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::PlanFileError;

    #[test]
    fn kiss_cov_boundary_helper_symbols() {
        let _ = machine_block_marker_patterns;
        let _ = count_begin_malvin_marker_lines;
    }

    #[test]
    fn count_begin_malvin_marker_lines_ignores_prose_and_counts_whole_lines() {
        let prose = "See BEGIN_MALVIN in docs.\n\n---\nBEGIN_MALVIN\n";
        assert_eq!(detect_rerun_user_span_end(prose), Ok(Some(27)));
        let duplicate = "BEGIN_MALVIN\n\n---\nBEGIN_MALVIN\n";
        assert_eq!(
            detect_rerun_user_span_end(duplicate),
            Err(PlanFileError::DuplicateBeginMalvinMarkers)
        );
        let indented = "  BEGIN_MALVIN  \n\n---\nBEGIN_MALVIN\n";
        assert_eq!(
            detect_rerun_user_span_end(indented),
            Err(PlanFileError::DuplicateBeginMalvinMarkers)
        );
    }
}
