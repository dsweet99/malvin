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
        router_a.contains("KPop: Find unsatisfied requirements")
            && router_a.contains("__MALVIN_DONE__")
            && !router_a.to_ascii_lowercase().contains("falsif"),
        "router_a should ask for unsatisfied requirements without falsification language"
    );
    for name in ["router_a.md", "router_b.md", "router_b_creative.md"] {
        let body = crate::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing {name}"))
            .to_ascii_lowercase();
        assert!(body.contains("kpop:"), "{name} must keep KPop steering");
        for needle in router_metaphors {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
    let router_b = crate::prompts::default_file("router_b.md").expect("router_b");
    let creative = crate::prompts::default_file("router_b_creative.md").expect("router_b_creative");
    assert!(
        router_b.contains("KPop: Satisfy the requirements.")
            && !router_b.contains("MBC2"),
        "router_b must keep KPop satisfy instruction without MBC2"
    );
    assert!(
        creative.contains("KPop: Satisfy the requirements.") && creative.contains("MBC2"),
        "router_b_creative must keep KPop satisfy instruction and MBC2"
    );
}
