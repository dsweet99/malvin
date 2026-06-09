#[cfg(unix)]
use super::unix_process_group_ps::{ProcRow, INIT_PID, list_proc_rows};

#[cfg(unix)]
pub(crate) fn pid_is_ancestor(ancestor: u32, mut current: u32, rows: &[ProcRow]) -> bool {
    let mut visited = std::collections::HashSet::new();
    loop {
        if current == ancestor {
            return true;
        }
        if !visited.insert(current) {
            break;
        }
        let Some(ppid) = rows.iter().find(|row| row.pid == current).map(|row| row.ppid) else {
            break;
        };
        if ppid <= INIT_PID {
            break;
        }
        current = ppid;
    }
    false
}

#[cfg(unix)]
pub(crate) fn holder_is_ancestor_of_process(holder_pid: u32) -> bool {
    let self_pid = std::process::id();
    if holder_pid == self_pid {
        return true;
    }
    let Some(rows) = list_proc_rows() else {
        return false;
    };
    pid_is_ancestor(holder_pid, self_pid, &rows)
}

#[cfg(not(unix))]
pub(crate) fn holder_is_ancestor_of_process(_: u32) -> bool {
    false
}
