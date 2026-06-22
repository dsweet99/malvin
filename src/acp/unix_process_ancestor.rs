#[cfg(unix)]
use super::unix_process_group_ps::{list_proc_rows, ProcRow};

#[cfg(unix)]
pub(crate) fn is_ancestor_pid(ancestor: u32, pid: u32) -> bool {
    if ancestor == pid {
        return true;
    }
    let Some(rows) = list_proc_rows() else {
        return false;
    };
    is_ancestor_pid_from_rows(&rows, ancestor, pid)
}

#[cfg(unix)]
pub(crate) fn is_ancestor_pid_from_rows(rows: &[ProcRow], ancestor: u32, pid: u32) -> bool {
    if ancestor == pid {
        return true;
    }
    let parent_of_pid: std::collections::HashMap<u32, u32> =
        rows.iter().map(|row| (row.pid, row.ppid)).collect();
    let mut current = pid;
    loop {
        let Some(&parent) = parent_of_pid.get(&current) else {
            return false;
        };
        if parent == 0 || parent == current {
            return false;
        }
        if parent == ancestor {
            return true;
        }
        current = parent;
    }
}
