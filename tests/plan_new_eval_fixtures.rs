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
    let plan = std::fs::read_to_string("plan_new_eval.md")
        .expect("plan_new_eval.md should exist and be readable");
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
        "plan_new_eval should reference at least one evaluations/*.sh harness"
    );
    for path in references {
        assert!(
            Path::new(&path).is_file(),
            "plan_new_eval references missing harness: {path}"
        );
    }
}
