use super::{DM_END, DM_START, feed_do_dm_stdout_text, reset_do_dm_filter};
use crate::output::{
    do_dm_stdout_mode, enable_stdout_capture, set_do_dm_stdout_mode, take_captured_stdout,
};

fn with_dm_capture(f: impl FnOnce()) -> String {
    let _dm = crate::output::DO_DM_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::reset_dm_hook_state();
    reset_do_dm_filter();
    set_do_dm_stdout_mode(true);
    enable_stdout_capture();
    f();
    let out = take_captured_stdout();
    set_do_dm_stdout_mode(false);
    reset_do_dm_filter();
    out
}
#[test]
fn extracts_body_between_fences_and_ignores_text_outside_fences() {
    {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("{DM_START}\nhello\n{DM_END}\n"));
        });
        assert_eq!(out, "hello");
    }
    {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text("noise\n");
            feed_do_dm_stdout_text(&format!("{DM_START}\nonly\n{DM_END}\n"));
        });
        assert_eq!(out, "only");
    }
}

#[test]
fn streaming_chunks_across_markers() {
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text("__MALVIN_DM_");
        feed_do_dm_stdout_text("START__\nxi\n__MALVIN_DM_");
        feed_do_dm_stdout_text("END__\n");
    });
    assert_eq!(out, "xi");
}

#[test]
fn mode_off_is_noop() {
        let _dm = crate::output::DO_DM_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set_do_dm_stdout_mode(false);
    reset_do_dm_filter();
    enable_stdout_capture();
    feed_do_dm_stdout_text(&format!("{DM_START}\nz\n{DM_END}\n"));
    assert!(take_captured_stdout().is_empty());
    assert!(!do_dm_stdout_mode());
}

#[test]
fn streaming_multibyte_outside_fences_does_not_panic() {
    const OUTSIDE: &str = "Obtain the current local date and time (with timezone name or numeric offset) and report it clearly to the user—prefer a";
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text(OUTSIDE);
        feed_do_dm_stdout_text(" café 日本語\n");
        feed_do_dm_stdout_text(&format!("{DM_START}\nbody—ok\n{DM_END}\n"));
    });
    assert_eq!(out, "body—ok");
}

#[test]
fn streaming_multibyte_inside_incomplete_end_does_not_panic() {
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text(&format!("{DM_START}\nline—one\n"));
        feed_do_dm_stdout_text("line—two\n");
        feed_do_dm_stdout_text(&format!("{DM_END}\n"));
    });
    assert!(out.contains("line—one"), "out={out:?}");
    assert!(out.contains("line—two"), "out={out:?}");
}

#[test]
fn rejects_same_line_start_and_end() {
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text(&format!("{DM_START} hello {DM_END}\n"));
        feed_do_dm_stdout_text(&format!("{DM_START}\nreal\n{DM_END}\n"));
    });
    assert_eq!(out, "real");
}

#[test]
fn rejects_non_alone_marker_forms() {
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text(&format!("x{DM_START}\nnope\n{DM_END}\n"));
        feed_do_dm_stdout_text(&format!("{DM_START}x\nnope\n{DM_END}\n"));
        feed_do_dm_stdout_text(&format!("{DM_START}\nsee {DM_END}\n{DM_END}\n"));
        feed_do_dm_stdout_text(&format!("{DM_START}\nalone\n{DM_END}\n"));
    });
    assert_eq!(out, format!("see {DM_END}\nalone"));
}
#[test]
fn accepts_markers_with_surrounding_whitespace_and_streaming_leading_whitespace_before_start_marker()
 {
    {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text(&format!("  {DM_START}\r\nspaced\n {DM_END} \n"));
        });
        assert_eq!(out, "spaced");
    }
    {
        let out = with_dm_capture(|| {
            feed_do_dm_stdout_text("  ");
            feed_do_dm_stdout_text(&format!("{DM_START}\nchunked\n{DM_END}\n"));
        });
        assert_eq!(out, "chunked");
    }
}

#[test]
fn rejects_start_and_body_on_one_line() {
    let out = with_dm_capture(|| {
        feed_do_dm_stdout_text(&format!("{DM_START} hello\n{DM_END}\n"));
        feed_do_dm_stdout_text(&format!("{DM_START}\nok\n{DM_END}\n"));
    });
    assert_eq!(out, "ok");
}
