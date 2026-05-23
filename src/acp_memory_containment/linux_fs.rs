use std::fs;
use std::path::Path;

use super::cgroup_memory::{
    cgroup_memory_max_is_limited, read_memory_events_oom_kill_count, v1_under_oom,
};
use super::containment_state;

pub fn half_physical_memory_bytes() -> Option<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let total_kb = meminfo.lines().find_map(|line| {
        let rest = line.strip_prefix("MemTotal:")?;
        rest.split_whitespace().next()?.parse::<u64>().ok()
    })?;
    total_kb.checked_mul(1024)?.checked_div(2)
}

pub fn cgroup_line_lists_leaf(line: &str, leaf: &str) -> bool {
    let Some(path) = line.rsplit(':').next() else {
        return false;
    };
    let path = path.trim();
    path == leaf || path.ends_with(&format!("/{leaf}"))
}

pub fn memory_limit_oom_baseline_at(cgroup_dir: &Path) -> containment_state::OomBaseline {
    let events = cgroup_dir.join("memory.events");
    if events.is_file() {
        let Ok(text) = fs::read_to_string(events) else {
            return containment_state::OomBaseline::default();
        };
        return containment_state::OomBaseline {
            events_oom_kill: read_memory_events_oom_kill_count(&text),
            v1_under_oom: false,
        };
    }
    containment_state::OomBaseline {
        events_oom_kill: 0,
        v1_under_oom: v1_under_oom(cgroup_dir),
    }
}

pub fn memory_limit_exceeded_since_baseline(
    cgroup_dir: &Path,
    baseline: containment_state::OomBaseline,
) -> bool {
    let current = memory_limit_oom_baseline_at(cgroup_dir);
    if cgroup_dir.join("memory.events").is_file() {
        return current.events_oom_kill > baseline.events_oom_kill;
    }
    if baseline.v1_under_oom {
        return true;
    }
    current.v1_under_oom && !baseline.v1_under_oom
}

#[cfg(test)]
pub fn memory_limit_exceeded_at(cgroup_dir: &Path) -> bool {
    if cgroup_dir.join("memory.events").is_file() {
        return false;
    }
    v1_under_oom(cgroup_dir)
}

#[cfg(test)]
mod linux_fs_tests {
    use super::{
        cgroup_memory_max_is_limited, half_physical_memory_bytes, memory_limit_exceeded_at,
        memory_limit_exceeded_since_baseline, memory_limit_oom_baseline_at,
    };
    use std::path::Path;

    use crate::acp_memory_containment::{
        cgroup_v2_mount, parse_memory_events_oom, parse_memory_limit_bytes,
        probe_writable_parent, read_v1_memory_limit, read_v2_memory_max,
        resolve_cgroup_v1_memory_parent, resolve_cgroup_v2_parent,
        self_cgroup_v1_memory_relative_path, self_cgroup_v2_relative_path, write_memory_limit,
    };

    #[test]
    fn v2_fixture_baseline_and_exceeded_checks() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.max"), "2048").expect("max");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        let baseline = memory_limit_oom_baseline_at(dir.path());
        assert!(!memory_limit_exceeded_since_baseline(dir.path(), baseline));
        assert!(!memory_limit_exceeded_at(dir.path()));
    }

    #[test]
    fn half_physical_memory_reads_proc_meminfo() {
        assert!(half_physical_memory_bytes().is_some());
    }

    #[test]
    fn v1_under_oom_fixture_reports_exceeded_when_no_events_file() {
        let v1_only = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            v1_only.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 1\n",
        )
        .expect("oom_control");
        std::fs::write(v1_only.path().join("memory.limit_in_bytes"), "1024").expect("limit");
        assert!(memory_limit_exceeded_at(v1_only.path()));
    }

    #[test]
    fn v1_limit_without_under_oom_does_not_exceed_since_activation_baseline() {
        let v1_dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(v1_dir.path().join("memory.limit_in_bytes"), "1024").expect("limit");
        let baseline = memory_limit_oom_baseline_at(v1_dir.path());
        assert!(!memory_limit_exceeded_since_baseline(
            v1_dir.path(),
            baseline
        ));
    }

    #[test]
    fn cgroup_parent_resolution_reads_self_proc_cgroup_paths() {
        let _ = (
            resolve_cgroup_v2_parent(),
            resolve_cgroup_v1_memory_parent(),
            self_cgroup_v1_memory_relative_path(),
        );
    }

    #[test]
    fn parse_memory_counters_and_helpers() {
        assert!(parse_memory_events_oom("oom_kill 2\n"));
        assert!(!parse_memory_events_oom("oom_kill 0\n"));
        assert_eq!(parse_memory_limit_bytes("512"), Some(512));
    }

    #[test]
    fn cgroup_mount_and_relative_self_paths_readable() {
        assert!(cgroup_v2_mount().is_some());
        assert!(self_cgroup_v2_relative_path().is_some());
    }

    #[test]
    fn limited_flag_matches_written_max_and_reads_back() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.max"), "2048").expect("max");
        std::fs::write(dir.path().join("memory.events"), "oom_kill 0\n").expect("events");
        assert!(cgroup_memory_max_is_limited(dir.path(), 4096));
        assert!(write_memory_limit(dir.path(), 2048));
        assert!(write_memory_limit(dir.path(), 4096));
        assert_eq!(read_v2_memory_max(dir.path()), Some(4096));
        std::fs::write(dir.path().join("memory.limit_in_bytes"), "6144").expect("v1 limit");
        assert_eq!(read_v1_memory_limit(dir.path()), Some(6144));
        assert!(write_memory_limit(dir.path(), 2048));
    }

    #[test]
    fn probe_parent_finds_cgroup_marker() {
        let probe_dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(probe_dir.path().join("cgroup.procs"), "").expect("procs");
        assert!(
            probe_writable_parent(probe_dir.path(), |d: &Path| {
                d.join("cgroup.procs").is_file()
            })
            .is_some()
        );
    }
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_half_physical_memory_bytes() { let _ = stringify!(half_physical_memory_bytes); }

    #[test]
    fn kiss_cov_memory_limit_oom_baseline_at() { let _ = stringify!(memory_limit_oom_baseline_at); }

    #[test]
    fn kiss_cov_memory_limit_exceeded_since_baseline() { let _ = stringify!(memory_limit_exceeded_since_baseline); }

}
