//! Tests for [`super::kpop_session`].

#[test]
fn gate_kpop_session_declared_solved_detects_kpop_solved_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("exp.md");
    std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
    assert!(super::run_loop::session_wrote_kpop_solved(&path).expect("read"));
}
