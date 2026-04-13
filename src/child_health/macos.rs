//! macOS: `proc_pidinfo` with `libproc::task_info::TaskAllInfo` (BSD `pbi_status` + `proc_taskinfo`
//! CPU totals and thread count).

use super::ChildHealth;
use errno::errno;
use libc::ESRCH;
use libproc::proc_pid::pidinfo;
use libproc::task_info::TaskAllInfo;
use std::time::Instant;

/// `SZOMB` — `bsd/sys/proc.h` `pbi_status` when the process is a zombie.
const P_STATUS_ZOMB: u32 = 5;

#[must_use]
pub(super) fn sample_child_health(pid: u32) -> ChildHealth {
    let Ok(pid_i) = i32::try_from(pid) else {
        return ChildHealth::cannot_sample();
    };
    if let Ok(info) = pidinfo::<TaskAllInfo>(pid_i, 0) {
        return child_health_from_sampled_task(&info);
    }
    if errno().0 == ESRCH {
        return ChildHealth::process_absent();
    }
    ChildHealth::cannot_sample()
}

/// Fields copied from [`TaskAllInfo`] for mapping into [`ChildHealth`] (keeps arity small for
/// helpers and tests).
#[derive(Clone, Copy)]
struct SampledTaskPidInfo {
    pbi_status: u32,
    pti_total_user: u64,
    pti_total_system: u64,
    pti_threadnum: i32,
    pti_csw: i32,
}

/// Maps fields from a successful `pidinfo::<TaskAllInfo>` sample into [`ChildHealth`].
#[must_use]
fn child_health_from_sampled_task(info: &TaskAllInfo) -> ChildHealth {
    child_health_from_pid_info_parts(&SampledTaskPidInfo {
        pbi_status: info.pbsd.pbi_status,
        pti_total_user: info.ptinfo.pti_total_user,
        pti_total_system: info.ptinfo.pti_total_system,
        pti_threadnum: info.ptinfo.pti_threadnum,
        pti_csw: info.ptinfo.pti_csw,
    })
}

#[must_use]
fn child_health_from_pid_info_parts(fields: &SampledTaskPidInfo) -> ChildHealth {
    let zombie = fields.pbi_status == P_STATUS_ZOMB;
    let cpu = fields
        .pti_total_user
        .saturating_add(fields.pti_total_system);
    let thread_count = u32::try_from(fields.pti_threadnum.max(0)).ok();
    let voluntary_ctxt = Some(u64::try_from(i64::from(fields.pti_csw).max(0)).unwrap_or(0));
    let state_hint = if zombie {
        Some('Z')
    } else {
        status_char_hint(fields.pbi_status)
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

#[must_use]
const fn status_char_hint(status: u32) -> Option<char> {
    match status {
        2 => Some('R'), // `SRUN`
        3 => Some('S'), // `SSLEEP`
        4 => Some('T'), // `SSTOP`
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_char_hint_maps_known_pbi_status_values() {
        let h = child_health_from_pid_info_parts(&SampledTaskPidInfo {
            pbi_status: 2,
            pti_total_user: 0,
            pti_total_system: 0,
            pti_threadnum: 1,
            pti_csw: 0,
        });
        assert_eq!(h.state_hint, Some('R'));
        assert!(!h.zombie);

        let h = child_health_from_pid_info_parts(&SampledTaskPidInfo {
            pbi_status: 3,
            pti_total_user: 0,
            pti_total_system: 0,
            pti_threadnum: 1,
            pti_csw: 0,
        });
        assert_eq!(h.state_hint, Some('S'));
        assert!(!h.zombie);

        let h = child_health_from_pid_info_parts(&SampledTaskPidInfo {
            pbi_status: 4,
            pti_total_user: 0,
            pti_total_system: 0,
            pti_threadnum: 1,
            pti_csw: 0,
        });
        assert_eq!(h.state_hint, Some('T'));
        assert!(!h.zombie);

        let h = child_health_from_pid_info_parts(&SampledTaskPidInfo {
            pbi_status: 99,
            pti_total_user: 0,
            pti_total_system: 0,
            pti_threadnum: 1,
            pti_csw: 0,
        });
        assert_eq!(h.state_hint, None);
        assert!(!h.zombie);
    }

    #[test]
    fn zombie_status_sets_zombie_and_z_state_hint() {
        let h = child_health_from_pid_info_parts(&SampledTaskPidInfo {
            pbi_status: 5,
            pti_total_user: 10,
            pti_total_system: 20,
            pti_threadnum: 2,
            pti_csw: 3,
        });
        assert!(h.zombie);
        assert_eq!(h.state_hint, Some('Z'));
        assert_eq!(h.cpu_time_total, 30);
        assert_eq!(h.thread_count, Some(2));
        assert_eq!(h.voluntary_ctxt, Some(3));
    }

    #[test]
    fn pid_out_of_i32_range_returns_cannot_sample_without_panicking() {
        let h = sample_child_health(u32::MAX);
        assert!(h.exists);
        assert!(!h.counters_trusted);
    }

    #[test]
    fn unlikely_pid_sample_has_absent_cannot_sample_or_live_shape() {
        // Very unlikely to exist; kernel should return `ESRCH` → `process_absent`, not panic.
        let h = sample_child_health(2_147_483_646);
        assert_ne!(
            (h.exists, h.counters_trusted),
            (false, false),
            "unexpected ChildHealth tuple: {h:?}"
        );
    }
}
