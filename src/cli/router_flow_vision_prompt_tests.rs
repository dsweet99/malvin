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
        "header_nokpop.md",
        "router_a_nokpop.md",
        "router_b_nokpop.md",
        "router_b_creative_nokpop.md",
        "kpop_common_fake.md",
        "router_summarize.md",
    ] {
        let body = crate::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing default prompt {name}"))
            .to_ascii_lowercase();
        for needle in vision_forbidden {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
    let router_a_nokpop = crate::prompts::default_file("router_a_nokpop.md").expect("router_a_nokpop");
    assert!(
        router_a_nokpop.contains("Regularization")
            && !router_a_nokpop.to_ascii_lowercase().contains("falsif")
            && !router_a_nokpop.contains("KPop:")
            && router_a_nokpop.contains("independent axes"),
        "router_a_nokpop should keep regularization without falsification/KPop"
    );
    for name in [
        "router_a_nokpop.md",
        "router_b_nokpop.md",
        "router_b_creative_nokpop.md",
    ] {
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
