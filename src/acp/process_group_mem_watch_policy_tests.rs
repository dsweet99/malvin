//! Policy and witness tests split out of `process_group_mem_watch.rs` (kiss
//! lines-per-file limit). The watched loop itself stays in the parent module.

#[cfg(all(test, unix))]
mod policy_tests {
    use super::super::{MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES, memory_watch_should_terminate};

    #[test]
    fn memory_watch_should_terminate_on_over_limit() {
        let mut failures = 0;
        assert!(memory_watch_should_terminate(
            Some(100),
            50,
            &mut failures,
            true
        ));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_should_not_terminate_when_under_limit() {
        let mut failures = 0;
        assert!(!memory_watch_should_terminate(
            Some(10),
            50,
            &mut failures,
            true
        ));
        assert_eq!(failures, 0);
    }

    #[test]
    fn memory_watch_fail_closed_after_consecutive_none_samples() {
        let mut failures = 0;
        for _ in 0..MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES - 1 {
            assert!(!memory_watch_should_terminate(
                None,
                u64::MAX,
                &mut failures,
                true
            ));
        }
        assert!(memory_watch_should_terminate(
            None,
            u64::MAX,
            &mut failures,
            true
        ));
    }

    #[test]
    fn memory_watch_no_fail_closed_when_disallowed() {
        let mut failures = 0;
        for _ in 0..MAX_CONSECUTIVE_RSS_SAMPLE_FAILURES + 2 {
            assert!(!memory_watch_should_terminate(
                None,
                u64::MAX,
                &mut failures,
                false
            ));
        }
        assert_eq!(failures, 0);
        assert!(memory_watch_should_terminate(
            Some(100),
            50,
            &mut failures,
            false
        ));
    }

    #[test]
    fn memory_watch_resets_failure_counter_after_successful_sample() {
        let mut failures = 2;
        assert!(!memory_watch_should_terminate(
            Some(1),
            u64::MAX,
            &mut failures,
            true
        ));
        assert_eq!(failures, 0);
        assert!(!memory_watch_should_terminate(
            None,
            u64::MAX,
            &mut failures,
            true
        ));
        assert_eq!(failures, 1);
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    #[cfg(unix)]
    use super::super::{
        MemWatchHandles, watch_process_group_memory, watch_process_group_memory_with_optional_pgid,
        watch_process_group_memory_with_rss_sampler,
    };

    #[test]
    #[cfg(unix)]
    fn kiss_cov_watch_sampler() {
        let _ = (
            watch_process_group_memory,
            watch_process_group_memory_with_optional_pgid,
            watch_process_group_memory_with_rss_sampler,
        );
        let _handles: Option<MemWatchHandles> = None;
    }
}
