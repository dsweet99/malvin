//! Linux: `/proc/<pid>/stat` and optional `/proc/<pid>/status`.

use super::ChildHealth;
use std::fs;
use std::io::ErrorKind;
use std::time::Instant;

pub(super) struct ParsedProcStat {
    pub state: u8,
    pub utime: u64,
    pub stime: u64,
    pub num_threads: u32,
}

#[must_use]
pub(super) fn parse_proc_stat_line(line: &str) -> Option<ParsedProcStat> {
    let after_comm = line.rsplit_once(')')?.1.trim_start();
    let mut it = after_comm.split_whitespace();
    let state = *it.next()?.as_bytes().first()?;
    for _ in 0..10 {
        it.next()?;
    }
    let utime: u64 = it.next()?.parse().ok()?;
    let stime: u64 = it.next()?.parse().ok()?;
    // Fields 16–19: `cutime`, `cstime`, `priority`, `nice`; field 20 is `num_threads`.
    for _ in 0..4 {
        it.next()?;
    }
    let num_threads: u32 = it.next()?.parse().ok()?;
    Some(ParsedProcStat {
        state,
        utime,
        stime,
        num_threads,
    })
}

#[must_use]
pub(super) fn parse_status_voluntary_ctxt(status: &str) -> Option<u64> {
    for line in status.lines() {
        if let Some(rest) = line.strip_prefix("voluntary_ctxt_switches:") {
            return rest.trim_start().parse().ok();
        }
    }
    None
}

#[must_use]
pub(super) fn sample_child_health(pid: u32) -> ChildHealth {
    let stat_path = format!("/proc/{pid}/stat");
    let raw = match fs::read_to_string(&stat_path) {
        Ok(s) => s,
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return ChildHealth::process_absent();
        }
        Err(_) => {
            return ChildHealth::cannot_sample();
        }
    };
    let Some(p) = parse_proc_stat_line(raw.trim_end()) else {
        return ChildHealth::cannot_sample();
    };
    let status_path = format!("/proc/{pid}/status");
    let voluntary_ctxt = fs::read_to_string(&status_path)
        .ok()
        .and_then(|s| parse_status_voluntary_ctxt(&s));
    let zombie = p.state == b'Z';
    ChildHealth {
        exists: true,
        zombie,
        state_hint: Some(p.state as char),
        counters_trusted: true,
        cpu_time_total: p.utime.saturating_add(p.stime),
        thread_count: Some(p.num_threads),
        voluntary_ctxt,
        sample_time: Instant::now(),
    }
}
