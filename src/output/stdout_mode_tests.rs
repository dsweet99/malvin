use super::{
    DO_DM_TEST_LOCK, StdoutMode, do_dm_stdout_markdown, do_dm_stdout_mode, enter_stdout_mode,
    heartbeat_stdout_suppressed, set_stdout_mode, stdout_display_discarded, stdout_mode,
};

fn lock() -> std::sync::MutexGuard<'static, ()> {
    DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn each_mode_implies_its_checks() {
    let _lock = lock();
    let cases = [
        (StdoutMode::Normal, false, false, false, false),
        (StdoutMode::DmOnly { markdown: false }, true, false, true, false),
        (StdoutMode::DmOnly { markdown: true }, true, true, true, false),
        (StdoutMode::Silent, false, false, true, true),
    ];
    for (mode, dm, markdown, quiet_heartbeat, discarded) in cases {
        set_stdout_mode(mode);
        assert_eq!(stdout_mode(), mode);
        assert_eq!(do_dm_stdout_mode(), dm, "{mode:?}");
        assert_eq!(do_dm_stdout_markdown(), markdown, "{mode:?}");
        assert_eq!(heartbeat_stdout_suppressed(), quiet_heartbeat, "{mode:?}");
        assert_eq!(stdout_display_discarded(), discarded, "{mode:?}");
    }
    set_stdout_mode(StdoutMode::Normal);
}

#[test]
fn a_guard_restores_the_whole_previous_mode_including_markdown() {
    let _lock = lock();
    let markdown = StdoutMode::DmOnly { markdown: true };
    set_stdout_mode(markdown);
    {
        let _normal = enter_stdout_mode(StdoutMode::Normal);
        assert_eq!(stdout_mode(), StdoutMode::Normal);
        {
            let _silent = enter_stdout_mode(StdoutMode::Silent);
            assert_eq!(stdout_mode(), StdoutMode::Silent);
        }
        assert_eq!(stdout_mode(), StdoutMode::Normal);
    }
    assert_eq!(stdout_mode(), markdown);
    set_stdout_mode(StdoutMode::Normal);
}
