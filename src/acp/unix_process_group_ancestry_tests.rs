#![cfg(all(test, unix))]

use super::unix_process_group_ancestry::pid_is_ancestor;
use super::unix_process_group_ps::ProcRow;

#[test]
fn pid_is_ancestor_follows_ppid_chain() {
    let rows = vec![
        ProcRow {
            pid: 10,
            pgid: 10,
            ppid: 1,
        },
        ProcRow {
            pid: 20,
            pgid: 20,
            ppid: 10,
        },
        ProcRow {
            pid: 30,
            pgid: 30,
            ppid: 20,
        },
    ];
    assert!(pid_is_ancestor(10, 30, &rows));
    assert!(!pid_is_ancestor(10, 1, &rows));
}
