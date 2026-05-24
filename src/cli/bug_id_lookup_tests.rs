use crate::cli::bug_id_lookup::{ensure_exp_log_solved, is_valid_bug_id, lookup_bug_id, validate_bug_id};
use crate::malvin_short_id;
use crate::output::{format_log_tag_inner, MALVIN_WHO};

#[test]
fn malvin_short_id_matches_charset() {
    for _ in 0..32 {
        let id = malvin_short_id();
        assert!(is_valid_bug_id(&id), "bad id: {id}");
    }
}

#[test]
fn validate_bug_id_rejects_bad() {
    assert!(validate_bug_id("Ma1b2").is_err());
    assert!(validate_bug_id("ma1b2c").is_err());
    assert!(validate_bug_id("Ma1b2C").is_err());
}

#[test]
fn lookup_finds_unique_bug_log_line() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    let run_dir = cwd.join("_malvin").join("20260101_abc");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let exp = run_dir.join("_kpop").join("exp_log_20260101_abc.md");
    std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir kpop");
    std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write exp");
    let rel = "./_malvin/20260101_abc/_kpop/exp_log_20260101_abc.md";
    std::fs::write(
        run_dir.join("stdout.log"),
        format!(
            "20260101.000000.000 [{}] BUG_LOG: Ma1b2c {rel}\n",
            format_log_tag_inner(MALVIN_WHO)
        ),
    )
    .expect("stdout");
    let resolved = lookup_bug_id(cwd, "Ma1b2c").expect("lookup");
    assert_eq!(resolved.run_dir, run_dir);
    assert_eq!(resolved.exp_log_path, exp);
}

#[test]
fn lookup_duplicate_ids_errors_with_two_runs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    for name in ["run_a", "run_b"] {
        let run_dir = cwd.join("_malvin").join(name);
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(
            run_dir.join("stdout.log"),
            format!(
                "20260101.000000.000 [{}] BUG_LOG: Mdup01 ./x\n",
                format_log_tag_inner(MALVIN_WHO)
            ),
        )
        .expect("stdout");
    }
    let err = lookup_bug_id(cwd, "Mdup01").unwrap_err();
    assert!(err.contains("ambiguous"), "got: {err}");
    assert!(err.contains("run_a"), "got: {err}");
    assert!(err.contains("run_b"), "got: {err}");
}

#[test]
fn lookup_not_found() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("_malvin")).expect("mkdir");
    let err = lookup_bug_id(tmp.path(), "Mnope1").unwrap_err();
    assert!(err.contains("no BUG_ID"), "got: {err}");
}

#[test]
fn lookup_bug_id_only_fallback() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    let run_dir = cwd.join("_malvin").join("20260102_xyz");
    std::fs::create_dir_all(run_dir.join("_kpop")).expect("mkdir");
    let exp = run_dir.join("_kpop").join("exp_log_20260102_xyz.md");
    std::fs::write(&exp, "## KPOP_SOLVED\n").expect("exp");
    std::fs::write(
        run_dir.join("stdout.log"),
        format!(
            "20260101.000000.000 [{}] BUG_ID: Monly1\n",
            format_log_tag_inner(MALVIN_WHO)
        ),
    )
    .expect("stdout");
    let resolved = lookup_bug_id(cwd, "Monly1").expect("fallback");
    assert_eq!(resolved.exp_log_path, exp);
}

#[test]
fn ensure_exp_log_solved_rejects_missing_marker() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let exp = tmp.path().join("exp.md");
    std::fs::write(&exp, "still looking\n").expect("write");
    assert!(ensure_exp_log_solved(&exp).is_err());
}

#[test]
fn lookup_rejects_untagged_bug_id_only_chatter_in_stdout_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    let run_dir = cwd.join("_malvin").join("20260104_agent");
    std::fs::create_dir_all(run_dir.join("_kpop")).expect("mkdir");
    let exp = run_dir.join("_kpop").join("exp_log_20260104_agent.md");
    std::fs::write(&exp, "## KPOP_SOLVED\n").expect("exp");
    std::fs::write(
        run_dir.join("stdout.log"),
        "tool: the user mentioned BUG_ID: Magnt1 but malvin never emitted it\n",
    )
    .expect("stdout");
    assert!(
        lookup_bug_id(cwd, "Magnt1").is_err(),
        "fix-by-id must not treat untagged agent chatter as an authoritative BUG_ID"
    );
}

#[test]
fn lookup_rejects_untagged_bug_log_line_in_stdout_log() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    let run_dir = cwd.join("_malvin").join("20260105_agent");
    std::fs::create_dir_all(run_dir.join("_kpop")).expect("mkdir");
    let exp = run_dir.join("_kpop").join("exp_log_20260105_agent.md");
    std::fs::write(&exp, "## KPOP_SOLVED\n").expect("exp");
    let rel = "./_malvin/20260105_agent/_kpop/exp_log_20260105_agent.md";
    std::fs::write(
        run_dir.join("stdout.log"),
        format!("acp trace: BUG_LOG: Mlogab {rel}\n"),
    )
    .expect("stdout");
    assert!(
        lookup_bug_id(cwd, "Mlogab").is_err(),
        "fix-by-id must not treat untagged BUG_LOG lines as authoritative"
    );
}

#[test]
fn lookup_rejects_missing_exp_log_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = tmp.path();
    let run_dir = cwd.join("_malvin").join("20260103_nope");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    std::fs::write(
        run_dir.join("stdout.log"),
        format!(
            "20260101.000000.000 [{}] BUG_LOG: Mbad01 ./_malvin/missing/exp_log_x.md\n",
            format_log_tag_inner(MALVIN_WHO)
        ),
    )
    .expect("stdout");
    assert!(lookup_bug_id(cwd, "Mbad01").is_err());
}
