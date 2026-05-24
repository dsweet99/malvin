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
fn sole_md_positional_targets_file_in_place() {
    let tmp = tempfile::tempdir().unwrap();
    let notes = tmp.path().join("notes.md");
    std::fs::write(&notes, "from file\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(notes.to_string_lossy().into_owned()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    assert_eq!(dest, notes);
    assert!(super::plan_resolve::is_sole_md_file_in_place_for_test(&plan));
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
        text: Some(input.to_string_lossy().into_owned()),
    };
    (tmp, dst_side, plan)
}

#[test]
fn sole_md_session_work_dir_matches_resolve_user_md_request() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::create_dir_all("nested").unwrap();
    std::fs::write("nested/in.md", "body\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some("nested/in.md".to_string()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    let nested_md = tmp.path().join("nested/in.md");
    assert_eq!(
        super::plan_resolve::plan_session_work_dir(&plan, &dest),
        crate::artifacts::work_dir_for_path(&nested_md),
    );
    std::env::set_current_dir(old_cwd).unwrap();
}

#[test]
fn plan_path_with_md_source_uses_destination_workspace() {
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
fn plan_path_same_file_as_md_source_skips_rewrite() {
    let tmp = tempfile::tempdir().unwrap();
    let plan_file = tmp.path().join("plan.md");
    std::fs::write(&plan_file, "keep\n\n").unwrap();
    let before = std::fs::read(&plan_file).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(plan_file.clone()),
        text: Some(plan_file.to_string_lossy().into_owned()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("apply");
    assert_eq!(std::fs::read(&plan_file).unwrap(), before);
}

#[test]
fn plan_path_flag_with_md_source_writes_copy() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("in.md");
    let output = tmp.path().join("out.md");
    std::fs::write(&input, "copy me\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(output.clone()),
        text: Some(input.to_string_lossy().into_owned()),
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
fn bare_at_positional_writes_literal_to_default_plan() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some("@".to_string()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("write");
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "@\n");
    std::env::set_current_dir(old_cwd).unwrap();
}

#[test]
fn nonexistent_md_positional_is_literal() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some("no_such_plan.md".to_string()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("write");
    assert_eq!(
        std::fs::read_to_string(dest).unwrap(),
        "no_such_plan.md\n"
    );
    std::env::set_current_dir(old_cwd).unwrap();
}

#[test]
fn plan_path_with_md_source_copies_trim_empty_file_body() {
    let tmp = tempfile::tempdir().unwrap();
    let input = tmp.path().join("empty.md");
    let output = tmp.path().join("out.md");
    std::fs::write(&input, "\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: Some(output.clone()),
        text: Some(input.to_string_lossy().into_owned()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("copy");
    assert_eq!(std::fs::read(&output).unwrap(), b"\n");
}

#[test]
fn sole_md_empty_existing_file_reviews_in_place_without_rewrite() {
    let tmp = tempfile::tempdir().unwrap();
    let empty = tmp.path().join("empty.md");
    std::fs::write(&empty, "\n").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some(empty.to_string_lossy().into_owned()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    assert_eq!(dest, empty);
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("in-place review skips rewrite");
    assert_eq!(std::fs::read(&empty).unwrap(), b"\n");
}

#[test]
fn md_directory_path_is_literal() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    std::fs::create_dir_all("notes.md").unwrap();
    let plan = crate::cli::PlanArgs {
        plan_path: None,
        text: Some("notes.md".to_string()),
    };
    let dest = super::plan_resolve::resolve_plan_destination(&plan).expect("dest");
    super::plan_resolve::apply_plan_source(&plan, &dest).expect("write");
    assert_eq!(std::fs::read_to_string(dest).unwrap(), "notes.md\n");
    std::env::set_current_dir(old_cwd).unwrap();
}
