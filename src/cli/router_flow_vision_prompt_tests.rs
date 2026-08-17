#[test]
fn default_router_prompts_follow_vision_problem_solving_language() {
    // VISION: no explicit coding/eval-task language; no soft coding-shaped metaphors in routers.
    let vision_forbidden = [
        "coding",
        "evaluation task",
        "evaluation tasks",
        "fast task",
        "fast_tasks",
    ];
    let router_metaphors = [
        "convenience check",
        "smoke check",
        "green light",
        "looks green",
        "proxy green",
        "stage then commit",
        "validate before mutate",
        "import",
        "module",
        "backend",
        "grader",
        "pytest",
    ];
    for name in [
        "header.md",
        "router_a.md",
        "router_b.md",
        "router_b_creative.md",
        "kpop_common.md",
        "router_summarize.md",
    ] {
        let body = crate::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing default prompt {name}"))
            .to_ascii_lowercase();
        for needle in vision_forbidden {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
    let router_a = crate::prompts::default_file("router_a.md").expect("router_a");
    assert!(
        router_a.contains("Regularization")
            && router_a.contains("falsif")
            && router_a.contains("independent axes"),
        "router_a should keep regularization, falsification, and matrix inference"
    );
    for name in ["router_a.md", "router_b.md", "router_b_creative.md"] {
        let body = crate::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing {name}"))
            .to_ascii_lowercase();
        assert!(
            body.contains("older analogue for the same abstract role")
                && body.contains("prefer the redefined policy")
                && body.contains("classify the ambiguity"),
            "{name} must preserve measured regularization rules"
        );
        for needle in router_metaphors {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
}
