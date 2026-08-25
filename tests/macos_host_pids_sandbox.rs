#![cfg(target_os = "macos")]

use std::collections::HashSet;

use malvin::process_group_rss::pids_sandbox_bytes;

#[test]
fn macos_host_pids_sandbox_bytes_positive() {
    let mut pids = HashSet::new();
    pids.insert(std::process::id());
    let bytes = pids_sandbox_bytes(&pids).expect("macos sandbox");
    assert!(bytes > 0);
}
