use malvin::output::{enable_stdout_capture, take_captured_stdout};
use malvin::test_support::test_utils::with_isolated_home;

use super::refresh_models_catalog;
use crate::cli::refresh_notice::REFRESHING_STDOUT;

enum RefreshCase {
    Automatic,
    Forced,
    Skip,
}

fn assert_refresh(case: RefreshCase) {
    let force = matches!(case, RefreshCase::Forced);
    let expect_run = !matches!(case, RefreshCase::Skip);
    let mut ran = false;
    enable_stdout_capture();
    refresh_models_catalog(force, || ran = true);
    let out = take_captured_stdout();
    assert_eq!(ran, expect_run, "{out}");
    assert_eq!(
        out.contains(REFRESHING_STDOUT),
        matches!(case, RefreshCase::Automatic),
        "{out}"
    );
}

#[test]
fn automatic_models_refresh_prints_refreshing_and_forced_refresh_stays_quiet() {
    with_isolated_home(|_| {
        assert_refresh(RefreshCase::Automatic);
        assert_refresh(RefreshCase::Forced);
        let now = malvin::clock::unix_now_secs();
        super::models_cmd_refresh::save_last_refresh_secs(now).expect("save");
        assert_refresh(RefreshCase::Skip);
        assert_refresh(RefreshCase::Forced);
    });
}
