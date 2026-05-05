use std::collections::HashSet;
use std::path::{self, Path};

const TRIM_CHARS: &[char] = &[
    '`', ',', '\n', '\r', ';', ')', '(', '"', '\'', '.', ':', '!', '?', '[', ']',
];

#[cfg_attr(unix, test)]
fn plan_new_eval_referenced_eval_harnesses_exist() {
    if Path::new("plan_new_eval.md").exists() {
        assert!(
            plan_references_present(Path::new("plan_new_eval.md")),
            "plan_new_eval.md must reference at least one valid evaluations/*.sh harness"
        );
    }
}

fn plan_references_present(plan_path: &Path) -> bool {
    let Ok(plan) = std::fs::read_to_string(plan_path) else {
        return false;
    };
    let mut references = HashSet::new();
    for token in plan.split_whitespace() {
        let mut token = token.trim().to_string();
        if let Some(link_start) = token.find("](") {
            if let Some(end) = token.rfind(')') {
                token = token[link_start + 2..end].to_string();
            }
        }
        let mut candidate = token.trim_matches(TRIM_CHARS).to_string();
        if let (Some(start), Some(end)) = (candidate.find('('), candidate.rfind(')')) {
            if start < end {
                candidate = candidate[start + 1..end].to_string();
            }
        }
        if !candidate.starts_with("evaluations/") {
            continue;
        }
        if path::Path::new(&candidate)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sh"))
        {
            references.insert(candidate.clone());
        }
    }
    assert!(
        !references.is_empty(),
        "plan_new_eval.md must reference at least one evaluations/*.sh harness"
    );
    for path in references {
        assert!(
            Path::new(&path).is_file(),
            "plan_new_eval references missing harness: {path}"
        );
    }
    true
}

fn setup_tmp_plan_with_markdown_eval_link(tmp: &tempfile::TempDir) -> std::path::PathBuf {
    let plan = tmp.path().join("plan_new_eval.md");
    let rel = "evaluations/markdown_link.sh";
    std::fs::create_dir_all(tmp.path().join("evaluations")).unwrap();
    std::fs::write(&plan, format!("Run with [markdown link]({rel})\n")).unwrap();
    std::fs::write(tmp.path().join(rel), "#!/usr/bin/env sh\nexit 0\n").unwrap();
    plan
}

#[cfg_attr(unix, test)]
fn plan_new_eval_referenced_eval_harnesses_exist_should_fail_when_plan_missing() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    assert!(!Path::new(&tmp.path().join("plan_new_eval.md")).exists());
    assert!(
        !plan_references_present(tmp.path().join("plan_new_eval.md").as_path()),
        "missing plan_new_eval.md should be detected by reference-presence helper"
    );
}

#[cfg_attr(unix, test)]
fn plan_new_eval_references_markdown_link_form_harness() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let plan = setup_tmp_plan_with_markdown_eval_link(&tmp);
    let old_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();
    let present = plan_references_present(&plan);
    std::env::set_current_dir(old_cwd).unwrap();
    assert!(present);
}
