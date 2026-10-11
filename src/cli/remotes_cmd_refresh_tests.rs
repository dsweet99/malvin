use malvin::modal_run::gpu_types::{GpuTypesRecord, gpu_types_cache_path, save_record};
use malvin::output::{enable_stdout_capture, take_captured_stdout};
use malvin::test_support::test_utils::with_isolated_home;

use super::{GpuTypes, RemotesArgs, load_gpu_types_for_remotes, run_remotes};
use crate::cli::refresh_notice::REFRESHING_STDOUT;

enum NoticeCase {
    Automatic,
    Forced,
    Cached,
}

fn assert_notice(case: NoticeCase) {
    let force = matches!(case, NoticeCase::Forced);
    let mut ran = false;
    enable_stdout_capture();
    let _gpu = load_gpu_types_for_remotes(force, |_| {
        ran = true;
        GpuTypes::default()
    });
    let out = take_captured_stdout();
    assert!(ran, "{out}");
    assert_eq!(
        out.contains(REFRESHING_STDOUT),
        matches!(case, NoticeCase::Automatic),
        "{out}"
    );
}

fn write_cache(fetched_secs: u64) {
    save_record(
        &gpu_types_cache_path(),
        &GpuTypesRecord {
            fetched_secs,
            types: vec!["T4".to_string()],
        },
    )
    .expect("save gpu cache");
}

#[test]
fn automatic_remote_refresh_prints_refreshing_and_forced_refresh_stays_quiet() {
    with_isolated_home(|_| {
        assert_notice(NoticeCase::Automatic);
        assert_notice(NoticeCase::Forced);
        write_cache(malvin::clock::unix_now_secs());
        assert_notice(NoticeCase::Cached);
        write_cache(0);
        assert_notice(NoticeCase::Automatic);
        assert_notice(NoticeCase::Forced);
    });
}

#[test]
fn fresh_remote_cache_lists_without_refreshing() {
    with_isolated_home(|_| {
        write_cache(malvin::clock::unix_now_secs());
        enable_stdout_capture();
        run_remotes(&RemotesArgs::default());
        let out = take_captured_stdout();
        assert!(out.contains("GPU types: T4"), "{out}");
        assert!(!out.contains(REFRESHING_STDOUT), "{out}");
    });
}
