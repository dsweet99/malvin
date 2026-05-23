#[test]
fn rejects_whitespace_only_plan_text() {
    assert!(super::plan_resolve::normalized_plan_file_bytes(" \n\t ").is_err());
}

#[test]
fn normalized_plan_file_bytes_preserves_internal_whitespace() {
    let bytes = super::plan_resolve::normalized_plan_file_bytes("  hi  ").expect("non-empty plan");
    assert_eq!(String::from_utf8(bytes).unwrap(), "  hi  \n");
}

#[test]
fn normalizes_trailing_newlines_to_single_terminal_newline() {
    let bytes = super::plan_resolve::normalized_plan_file_bytes("a\n\n").expect("non-empty plan");
    assert_eq!(String::from_utf8(bytes).unwrap(), "a\n");
}

#[test]
fn sole_at_positional_targets_file_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let notes = tmp.path().join("notes.md");
    std::fs::write(&notes, "from file\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(format!("@{}", notes.display())),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    assert_eq!(dest, notes);
    assert!(super::plan_resolve::is_sole_at_in_place_for_test(&plan));
    assert!(
        super::plan_resolve::plan_source_bytes_for_test(&plan, &dest)
            .unwrap()
            .is_none()
    );
}

fn cross_dir_plan_copy_fixture() -> (tempfile::TempDir, std::path::PathBuf, crate::cli::PlanArgs) {
    let tmp = tempfile::tempdir().unwrap();
    let src_side = tmp.path().join("src_side");
    let dst_side = tmp.path().join("dst_side");
    std::fs::create_dir_all(&src_side).unwrap();
    std::fs::create_dir_all(&dst_side).unwrap();
    let input = src_side.join("in.md");
    let output = dst_side.join("out.md");
    std::fs::write(&input, "body\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(output),
        text: Some(format!("@{}", input.display())),
    };
    (tmp, dst_side, plan)
}

#[test]
fn sole_at_session_work_dir_matches_resolve_user_request() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("nested");
    std::fs::create_dir_all(&sub).unwrap();
    let input = sub.join("in.md");
    std::fs::write(&input, "body\n").unwrap();
    let at_arg = format!("@{}", input.display());
    let (_, expected) = crate::artifacts::resolve_user_request(&at_arg).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(at_arg),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    assert_eq!(super::plan_resolve::plan_session_work_dir(&plan, &dest), expected);
}

#[test]
fn plan_path_with_at_source_uses_destination_workspace() {
    let (_tmp, dst_side, plan) = cross_dir_plan_copy_fixture();
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("copy");
    let session_wd = super::plan_resolve::plan_session_work_dir(&plan, &dest);
    assert_eq!(session_wd, dst_side);
    let artifacts = crate::artifacts::create_run_artifacts(&dest, Some(session_wd.as_path()))
        .expect("artifacts");
    assert_eq!(artifacts.work_dir, dst_side);
    assert!(artifacts.run_dir.starts_with(dst_side.join("_malvin")));
}

#[test]
fn plan_path_same_file_as_at_source_skips_rewrite() {
    let tmp = tempfile::tempdir().unwrap();
    let plan_file = tmp.path().join("plan.md");
    std::fs::write(&plan_file, "keep\n\n").unwrap();
    let before = std::fs::read(&plan_file).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(plan_file.clone()),
        text: Some(format!("@{}", plan_file.display())),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("apply");
    assert_eq!(std::fs::read(&plan_file).unwrap(), before);
}

#[test]
fn plan_path_flag_with_at_source_writes_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("in.md");
    let output = tmp.path().join("out.md");
    std::fs::write(&input, "copy me\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(output.clone()),
        text: Some(format!("@{}", input.display())),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("write");
    assert_eq!(std::fs::read_to_string(&output).unwrap(), "copy me\n");
}

#[test]
fn literal_positional_writes_normalized_text() {
    let tmp = tempfile::tempdir().unwrap();
    let plan_file = tmp.path().join("plan.md");
    let plan = crate::cli::PlanArgs {
        plan_path: Some(plan_file.clone()),
        text: Some("  literal  ".to_string()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("write");
    assert_eq!(std::fs::read_to_string(&plan_file).unwrap(), "literal\n");
}

#[test]
fn bare_at_positional_rejected_like_resolve_user_request() {
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some("@".to_string()),
    };
    let err = super::plan_resolve::resolve_plan_destination(&plan).unwrap_err();
    assert_eq!(err, "Empty path after `@`.");
}

#[test]
fn bare_at_with_plan_path_is_rejected_on_copy() {
    let plan = crate::cli::PlanArgs {
        plan_path: Some(std::path::PathBuf::from("out.md")),
        text: Some("@".to_string()),
    };
    let err = super::plan_resolve::resolve_plan_destination(&plan).unwrap_err();
    assert_eq!(err, "Empty path after `@`.");
}

#[test]
fn plan_path_with_at_source_copies_trim_empty_file_body() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("empty.md");
    let output = tmp.path().join("out.md");
    std::fs::write(&input, "\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(output.clone()),
        text: Some(format!("@{}", input.display())),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("copy");
    assert_eq!(std::fs::read(&output).unwrap(), b"\n");
}

#[test]
fn sole_at_empty_existing_file_reviews_in_place_without_rewrite() {
    let tmp = tempfile::tempdir().unwrap();
    let empty = tmp.path().join("empty.md");
    std::fs::write(&empty, "\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(format!("@{}", empty.display())),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    assert_eq!(dest, empty);
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("in-place review skips rewrite");
    assert_eq!(std::fs::read(&empty).unwrap(), b"\n");
}

#[test]
fn sole_at_directory_is_rejected_before_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("plans");
    std::fs::create_dir_all(&dir).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(format!("@{}", dir.display())),
    };
    let err = super::plan_resolve::resolve_plan_destination(&plan).unwrap_err();
    assert!(
        err.contains("not a file") || err.contains("directory"),
        "unexpected err: {err}"
    );
}
