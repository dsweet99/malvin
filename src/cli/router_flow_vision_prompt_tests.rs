#[test]
fn default_router_prompt_skeletons_keep_template_keys() {
    for name in [
        "header.md",
        "router_a.md",
        "router_a_2.md",
        "router_a_audit.md",
        "router_b.md",
        "router_b_satisfy.md",
        "router_b_creative_lead.md",
        "router_b_done_note.md",
        "router_summarize.md",
        "do_header.md",
        "mbc2.md",
    ] {
        let body = malvin::prompts::default_file(name)
            .unwrap_or_else(|| panic!("missing default prompt {name}"));
        assert!(!body.trim().is_empty(), "{name} must stay embedded");
    }
    let router_a = malvin::prompts::default_file("router_a.md").expect("router_a");
    assert!(
        router_a.contains("{{ audit_directive }}")
            && router_a.contains("{{ code_extra }}")
            && router_a.contains("{{ user_request_path }}"),
        "router_a must keep the keys the router fills"
    );
    let router_b = malvin::prompts::default_file("router_b.md").expect("router_b");
    assert!(
        router_b.contains("{{ satisfy_line }}")
            && router_b.contains("{{ creative_lead }}")
            && router_b.contains("{{ done_note }}"),
        "router_b must keep the keys the router fills"
    );
}
