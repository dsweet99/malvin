//! **`KPopSoftConstraints`** — workflow-specific constraint prompt basenames under `default_prompts/`.

/// Basename of the soft-constraints prompt for a gate workflow command.
#[must_use]
pub fn constraints_basename(workflow: &str) -> Option<&'static str> {
    match workflow {
        "code" => Some("code_constraints.md"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::constraints_basename;

    #[test]
    fn constraints_basename_maps_gate_workflows() {
        assert_eq!(constraints_basename("code"), Some("code_constraints.md"));
        assert_eq!(constraints_basename("tidy"), None);
        assert_eq!(constraints_basename("delight"), None);
        assert_eq!(constraints_basename("explain"), None);
        assert_eq!(constraints_basename("kpop"), None);
    }
}
