use std::time::{Duration, Instant};

use super::*;

fn health_snapshot(t0: Instant, cpu: u64, threads: u32, ctxt: Option<u64>) -> ChildHealth {
    ChildHealth {
        exists: true,
        zombie: false,
        state_hint: Some('S'),
        counters_trusted: true,
        cpu_time_total: cpu,
        thread_count: Some(threads),
        voluntary_ctxt: ctxt,
        sample_time: t0,
    }
}

#[test]
fn health_progress_sees_cpu_or_ctxt_changes() {
    let t0 = Instant::now();
    assert!(health_indicates_progress(
        &health_snapshot(t0, 10, 3, Some(100)),
        &health_snapshot(t0, 11, 3, Some(100)),
    ));
    assert!(health_indicates_progress(
        &health_snapshot(t0, 5, 1, Some(10)),
        &health_snapshot(t0, 5, 1, Some(11)),
    ));
}

#[test]
fn silence_grace_clamps() {
    assert_eq!(
        silence_grace_for_rpc_timeout(Duration::from_secs(0)),
        Duration::from_millis(50)
    );
    assert_eq!(
        silence_grace_for_rpc_timeout(Duration::from_secs(100)),
        Duration::from_millis(250)
    );
}

#[test]
fn cannot_sample_is_not_treated_as_absent_process() {
    assert!(ChildHealth::cannot_sample().exists);
    assert!(!ChildHealth::process_absent().exists);
}

#[test]
fn evaluate_maps_cannot_sample_pair_to_hung_not_not_running() {
    let first = ChildHealth::cannot_sample();
    let second = ChildHealth::cannot_sample();
    assert_eq!(
        super::silence_outcome_from_pair(&first, &second),
        SilenceHealthOutcome::AppearsHung
    );
}

/// First read failed (`cannot_sample`): a later trusted snapshot cannot be compared for movement, so
/// we do not extend (avoids treating typical `/proc` fields as "progress" when the child may be hung).
#[test]
fn first_untrusted_second_trusted_does_not_infer_progress() {
    let t0 = Instant::now();
    let first = ChildHealth::cannot_sample();
    let second = health_snapshot(t0, 42, 1, Some(10_000));
    assert!(!health_indicates_progress(&first, &second));
    assert_eq!(
        super::silence_outcome_from_pair(&first, &second),
        SilenceHealthOutcome::AppearsHung
    );
}

#[test]
fn first_untrusted_second_trusted_zero_counters_still_no_progress() {
    let t0 = Instant::now();
    let first = ChildHealth::cannot_sample();
    let second = ChildHealth {
        exists: true,
        zombie: false,
        state_hint: None,
        counters_trusted: true,
        cpu_time_total: 0,
        thread_count: None,
        voluntary_ctxt: None,
        sample_time: t0,
    };
    assert!(!health_indicates_progress(&first, &second));
    assert_eq!(
        super::silence_outcome_from_pair(&first, &second),
        SilenceHealthOutcome::AppearsHung
    );
}

/// `ChildHealth::cannot_sample()` uses zero counters; comparing to a prior good sample must not
/// look like "progress" (CPU time went from N to 0), or we extend the RPC wait on I/O failure.
#[test]
fn silence_second_sample_io_failure_must_not_masquerade_as_progress() {
    let t0 = Instant::now();
    let first = health_snapshot(t0, 1000, 1, Some(99));
    let second = ChildHealth::cannot_sample();
    assert_eq!(
        super::silence_outcome_from_pair(&first, &second),
        SilenceHealthOutcome::AppearsHung,
        "failed second OS sample must not be treated as busy child via fake counter delta"
    );
}

#[cfg(target_os = "linux")]
mod linux_parse {
    use super::super::linux::{parse_proc_stat_line, parse_status_voluntary_ctxt};

    #[test]
    fn parses_proc_stat_with_parentheses_in_comm() {
        let line = "12345 (fake (name)) S 1 1 1 0 0 0 0 0 0 0 40 50 0 0 0 0 3";
        let p = parse_proc_stat_line(line).expect("parse");
        assert_eq!(p.state, b'S');
        assert_eq!(p.utime, 40);
        assert_eq!(p.stime, 50);
        assert_eq!(p.num_threads, 3);
    }

    #[test]
    fn voluntary_ctxt_parsed_from_status() {
        let s = "Name:\tfoo\nvoluntary_ctxt_switches:\t4242\n";
        assert_eq!(parse_status_voluntary_ctxt(s), Some(4242));
    }

    /// Value must tolerate a trailing `\r` on the line (e.g. odd line endings) so we do not drop
    /// the counter and under-count OS "progress" during silence grace.
    #[test]
    fn voluntary_ctxt_parses_when_value_has_trailing_cr() {
        let s = "voluntary_ctxt_switches:\t4242\r";
        assert_eq!(
            parse_status_voluntary_ctxt(s),
            Some(4242),
            "trailing \\r after the numeric value must not break u64 parse"
        );
    }
}

#[cfg(target_os = "macos")]
mod macos_sample;
