use std::collections::HashSet;
use std::path::{self, Path};

const TRIM_CHARS: &[char] = &[
    '`',
    ',',
    '\n',
    '\r',
    ';',
    ')',
    '(',
    '"',
    '\'',
    '.',
    ':',
    '!',
    '?',
];

#[test]
#[cfg(unix)]
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
        let candidate = token.trim_matches(TRIM_CHARS);
        if !candidate.starts_with("evaluations/") {
            continue;
        }
        if path::Path::new(candidate)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("sh"))
        {
            references.insert(candidate.to_string());
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

#[test]
#[cfg(unix)]
fn plan_new_eval_referenced_eval_harnesses_exist_should_fail_when_plan_missing() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    assert!(!Path::new(&tmp.path().join("plan_new_eval.md")).exists());
    assert!(
        !plan_references_present(tmp.path().join("plan_new_eval.md").as_path()),
        "missing plan_new_eval.md should be detected by reference-presence helper"
    );
}
