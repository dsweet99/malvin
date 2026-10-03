pub const MALVIN_DONE: &str = "__MALVIN_DONE__";
pub const DM_START: &str = "__MALVIN_DM_START__";
pub const DM_END: &str = "__MALVIN_DM_END__";

#[must_use]
pub fn is_sentinel_line(line: &str, marker: &str) -> bool {
    line.trim_ascii() == marker
}

#[must_use]
pub(crate) fn may_become_sentinel_line(partial: &str, marker: &str) -> bool {
    let partial = partial.trim_ascii_start();
    marker.starts_with(partial)
        || partial
            .strip_prefix(marker)
            .is_some_and(|rest| rest.trim_ascii().is_empty())
}

#[cfg(test)]
#[path = "sentinel_tests.rs"]
mod sentinel_tests;
