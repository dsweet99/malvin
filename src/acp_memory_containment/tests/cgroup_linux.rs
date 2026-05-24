mod fs_tests {
    use crate::acp_memory_containment::cgroup_memory_max_is_limited;
    use crate::acp_memory_containment::{
        memory_limit_exceeded_at, memory_limit_exceeded_since_baseline,
        memory_limit_oom_baseline_at,
    };
    use crate::acp_memory_containment::{
        cgroup_v2_mount, parse_memory_events_oom, parse_memory_limit_bytes,
        self_cgroup_v2_relative_path,
    };

    #[test]
    fn parse_memory_limit_bytes_accepts_digits() {
        assert_eq!(parse_memory_limit_bytes("1048576"), Some(1_048_576));
    }

    #[test]
    fn parse_memory_limit_bytes_rejects_max() {
        assert_eq!(parse_memory_limit_bytes("max"), None);
    }

    #[test]
    fn parse_memory_events_oom_detects_counter() {
        assert!(parse_memory_events_oom("oom_kill 3\n"));
        assert!(!parse_memory_events_oom("oom_kill 0\n"));
    }

    #[test]
    fn cgroup_v2_mount_returns_path_on_linux() {
        assert!(cgroup_v2_mount().is_some());
    }

    #[test]
    fn self_cgroup_v2_relative_path_returns_nonempty() {
        assert!(self_cgroup_v2_relative_path().is_some());
    }

    #[test]
    fn cgroup_memory_max_is_limited_reads_temp_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("memory.max"), "2097152").expect("write");
        assert!(cgroup_memory_max_is_limited(dir.path(), 2_097_152));
        assert!(!cgroup_memory_max_is_limited(dir.path(), 512 * 1024));
    }

    #[test]
    fn memory_limit_oom_baseline_and_exceeded_round_trip() {
        let v2_dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(v2_dir.path().join("memory.events"), "oom_kill 0\n").expect("write");
        let baseline = memory_limit_oom_baseline_at(v2_dir.path());
        assert!(!memory_limit_exceeded_since_baseline(
            v2_dir.path(),
            baseline
        ));
        assert!(!memory_limit_exceeded_at(v2_dir.path()));

        let v1_dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            v1_dir.path().join("memory.oom_control"),
            "oom_kill_disable 0\nunder_oom 1\n",
        )
        .expect("write");
        assert!(memory_limit_exceeded_at(v1_dir.path()));
    }

    #[test]
    fn verify_pid_in_cgroup_rejects_zero_pid() {
        use crate::acp_memory_containment::{CgroupSpawnPlan, verify_pid_in_cgroup};

        let dir = tempfile::tempdir().expect("tempdir");
        let plan = CgroupSpawnPlan {
            cgroup_dir: dir.path().to_path_buf(),
            memory_max_bytes: 1,
        };
        assert!(!verify_pid_in_cgroup(0, &plan));
    }
}
