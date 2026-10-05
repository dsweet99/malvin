use std::collections::HashSet;

use super::ProcRow;

pub(super) fn snapshot_pids_from_proc() -> Option<HashSet<u32>> {
    let mut pids = HashSet::new();
    for entry in std::fs::read_dir("/proc").ok()? {
        let Ok(entry) = entry else {
            continue;
        };
        if let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() {
            pids.insert(pid);
        }
    }
    Some(pids)
}

pub(super) fn list_proc_rows_from_proc() -> Option<Vec<ProcRow>> {
    let mut rows = Vec::new();
    for entry in std::fs::read_dir("/proc").ok()? {
        if let Some(row) = proc_row_from_dir_entry(entry.ok()?) {
            rows.push(row);
        }
    }
    Some(rows)
}

fn proc_row_from_dir_entry(entry: std::fs::DirEntry) -> Option<ProcRow> {
    let pid: u32 = entry.file_name().to_string_lossy().parse().ok()?;
    let stat = std::fs::read_to_string(entry.path().join("stat")).ok()?;
    let after_comm = stat.rsplit_once(") ")?.1;
    let mut fields = after_comm.split_whitespace();
    let _state = fields.next()?;
    let parent_pid: u32 = fields.next()?.parse().ok()?;
    let pgrp: u32 = fields.next()?.parse().ok()?;
    Some(ProcRow {
        pid,
        pgid: pgrp,
        ppid: parent_pid,
    })
}
