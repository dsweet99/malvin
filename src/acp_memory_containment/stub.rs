use super::AcpMemoryContainment;

#[must_use]
pub const fn inactive_containment() -> AcpMemoryContainment {
    AcpMemoryContainment::inactive()
}

#[cfg(test)]
mod stub_tests {
    use super::inactive_containment;

    #[test]
    fn inactive_containment_returns_inactive() {
        assert!(!inactive_containment().active());
    }
}
