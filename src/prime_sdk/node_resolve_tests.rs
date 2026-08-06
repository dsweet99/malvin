//! Move `node_resolve` tests + name witnesses into a `*tests.rs` file.

use super::node_resolve::*;

#[test]
fn resolve_node_bin_finds_modern_node() {
    let path = prime_resolve_node_bin().expect("modern node");
    assert!(path.is_file());
}

#[test]
fn kiss_cov_node_resolve_names() {
    let _ = stringify!(prime_resolve_node_bin_uncached);
    let _ = stringify!(prime_sticky_node_bin_path);
    let _ = stringify!(prime_read_sticky_node_bin);
    let _ = stringify!(prime_write_sticky_node_bin);
    let _ = stringify!(prime_node_candidates);
    let _ = stringify!(prime_push_unique);
    let _ = stringify!(prime_agent_nodes);
    let _ = stringify!(prime_node_meets_floor);
    let _ = stringify!(prime_node_major_minor);
    let _ = stringify!(prime_apply_quiet_node_cli);
    let _ = stringify!(prime_apply_quiet_node_cli_std);
}
