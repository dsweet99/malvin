// Non-Linux / non-macOS: no OS backend; delegates to `ChildHealth::cannot_sample`.

use super::ChildHealth;

#[must_use]
pub(super) fn sample_child_health_other(_pid: u32) -> ChildHealth {
    ChildHealth::cannot_sample()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
#[cfg(test)]
mod other_inline_tests {
    use super::sample_child_health_other;

    #[test]
    fn sample_other_returns_cannot_sample() {
        let h = sample_child_health_other(42);
        assert!(h.exists);
        assert!(!h.counters_trusted);
    }
}
