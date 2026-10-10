#![cfg(all(test, unix))]

#[test]
fn is_ancestor_pid_from_rows_walks_parent_chain() {
    let rows = vec![
        super::unix_process_group_ps::ProcRow {
            pid: 1,
            pgid: 1,
            ppid: 0,
        },
        super::unix_process_group_ps::ProcRow {
            pid: 10,
            pgid: 10,
            ppid: 1,
        },
        super::unix_process_group_ps::ProcRow {
            pid: 11,
            pgid: 10,
            ppid: 10,
        },
        super::unix_process_group_ps::ProcRow {
            pid: 12,
            pgid: 10,
            ppid: 11,
        },
        super::unix_process_group_ps::ProcRow {
            pid: 13,
            pgid: 10,
            ppid: 10,
        },
    ];
    assert!(super::unix_process_ancestor::is_ancestor_pid_from_rows(
        &rows, 10, 12
    ));
    assert!(super::unix_process_ancestor::is_ancestor_pid_from_rows(
        &rows, 11, 12
    ));
    assert!(super::unix_process_ancestor::is_ancestor_pid_from_rows(
        &rows, 10, 11
    ));
    assert!(!super::unix_process_ancestor::is_ancestor_pid_from_rows(
        &rows, 12, 10
    ));
    assert!(!super::unix_process_ancestor::is_ancestor_pid_from_rows(
        &rows, 11, 13
    ));
}
