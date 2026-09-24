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
        "router_a_audit.md",
        "router_b.md",
        "router_b_satisfy.md",
        "router_b_satisfy_brief.md",
        "router_b_creative_lead.md",
        "router_summarize.md",
    ] {
        let body = malvin::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing default prompt {name}"))
            .to_ascii_lowercase();
        for needle in vision_forbidden {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
    let router_a = malvin::prompts::default_file("router_a.md").expect("router_a");
    let router_a_audit =
        malvin::prompts::default_file("router_a_audit.md").expect("router_a_audit");
    assert!(
        router_a.contains("{{ audit_directive }}")
            && router_a.contains("__MALVIN_DONE__")
            && !router_a.to_ascii_lowercase().contains("falsif"),
        "router_a should keep shared audit structure without falsification language"
    );
    assert!(
        router_a_audit.contains("KPop: Find unsatisfied requirements"),
        "router_a_audit should ask for unsatisfied requirements"
    );
    for name in [
        "router_a_audit.md",
        "router_b_satisfy.md",
        "router_b_satisfy_brief.md",
    ] {
        let body = malvin::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing {name}"))
            .to_ascii_lowercase();
        assert!(body.contains("kpop:"), "{name} must keep KPop steering");
        for needle in router_metaphors {
            assert!(!body.contains(needle), "{name} must not contain {needle:?}");
        }
    }
    let router_b = malvin::prompts::default_file("router_b.md").expect("router_b");
    let satisfy = malvin::prompts::default_file("router_b_satisfy.md").expect("router_b_satisfy");
    let creative_lead =
        malvin::prompts::default_file("router_b_creative_lead.md").expect("creative_lead");
    let satisfy_brief =
        malvin::prompts::default_file("router_b_satisfy_brief.md").expect("satisfy_brief");
    assert!(
        router_b.contains("{{ satisfy_line }}")
            && router_b.contains("{{ creative_lead }}")
            && !router_b.contains("MBC2"),
        "router_b skeleton must use template keys without MBC2"
    );
    assert!(
        satisfy.contains("KPop: Satisfy the requirements.") && !satisfy.contains("MBC2"),
        "router_b_satisfy must keep KPop satisfy instruction without MBC2"
    );
    assert!(
        satisfy_brief.contains("KPop: Satisfy the requirements.")
            && creative_lead.contains("MBC2"),
        "creative fragments must keep KPop satisfy instruction and MBC2"
    );
}
