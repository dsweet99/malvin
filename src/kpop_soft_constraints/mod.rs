//! **`KPopSoftConstraints`** — workflow-specific constraint prompt basenames under `default_prompts/`.

/// Basename of the soft-constraints prompt for a gate workflow command.
#[must_use]
pub fn constraints_basename(workflow: &str) -> Option<&'static str> {
    match workflow {
        "code" => Some("code_constraints.md"),
        "tidy" => Some("tidy_constraints.md"),
        "explain" => Some("explain_constraints.md"),
        "revise" => Some("revise_constraints.md"),
        "delight" => Some("delight_constraints.md"),
        "init" => Some("init_constraints.md"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::constraints_basename;

    #[test]
    fn constraints_basename_maps_gate_workflows() {
        assert_eq!(constraints_basename("code"), Some("code_constraints.md"));
        assert_eq!(constraints_basename("tidy"), Some("tidy_constraints.md"));
        assert_eq!(constraints_basename("explain"), Some("explain_constraints.md"));
        assert_eq!(constraints_basename("revise"), Some("revise_constraints.md"));
        assert_eq!(constraints_basename("delight"), Some("delight_constraints.md"));
        assert_eq!(constraints_basename("init"), Some("init_constraints.md"));
        assert_eq!(constraints_basename("kpop"), None);
    }
}
