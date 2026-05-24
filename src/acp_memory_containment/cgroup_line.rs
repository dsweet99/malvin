#[must_use]
pub fn cgroup_line_lists_leaf(line: &str, leaf: &str) -> bool {
    let Some(path) = line.rsplit(':').next() else {
        return false;
    };
    let path = path.trim();
    path == leaf || path.ends_with(&format!("/{leaf}"))
}

#[cfg(test)]
mod tests {
    use super::cgroup_line_lists_leaf;

    #[test]
    fn cgroup_line_lists_leaf_matches_suffix() {
        assert!(cgroup_line_lists_leaf(
            "0::/malvin-acp-1-0",
            "malvin-acp-1-0"
        ));
        assert!(!cgroup_line_lists_leaf("0::/other", "malvin-acp-1-0"));
    }
}
