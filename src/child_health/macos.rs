//! macOS: `proc_pidinfo` with `libproc::task_info::TaskAllInfo` (BSD `pbi_status` + `proc_taskinfo`
//! CPU totals and thread count).

use super::ChildHealth;
use errno::errno;
use libproc::proc_pid::pidinfo;
use libproc::task_info::TaskAllInfo;
use libc::ESRCH;
use std::time::Instant;

/// `SZOMB` — `bsd/sys/proc.h` `pbi_status` when the process is a zombie.
const P_STATUS_ZOMB: u32 = 5;

#[must_use]
pub(super) fn sample_child_health(pid: u32) -> ChildHealth {
    let Ok(pid_i) = i32::try_from(pid) else {
        return ChildHealth::cannot_sample();
    };
    match pidinfo::<TaskAllInfo>(pid_i, 0) {
        Ok(info) => {
            let zombie = info.pbsd.pbi_status == P_STATUS_ZOMB;
            let cpu = info
                .ptinfo
                .pti_total_user
                .saturating_add(info.ptinfo.pti_total_system);
            let thread_count = u32::try_from(info.ptinfo.pti_threadnum.max(0)).ok();
            let voluntary_ctxt = Some(info.ptinfo.pti_csw.max(0) as u64);
            let state_hint = if zombie {
                Some('Z')
            } else {
                status_char_hint(info.pbsd.pbi_status)
            };
            ChildHealth {
                exists: true,
                zombie,
                state_hint,
                counters_trusted: true,
                cpu_time_total: cpu,
                thread_count,
                voluntary_ctxt,
                sample_time: Instant::now(),
            }
        }
        Err(_) => {
            if errno().0 == ESRCH as i32 {
                return ChildHealth::process_absent();
            }
            ChildHealth::cannot_sample()
        }
    }
}

#[must_use]
const fn status_char_hint(status: u32) -> Option<char> {
    match status {
        2 => Some('R'), // `SRUN`
        3 => Some('S'), // `SSLEEP`
        4 => Some('T'), // `SSTOP`
        _ => None,
    }
}
