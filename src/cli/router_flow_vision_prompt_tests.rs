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
    header_advice_bullets_do_not_mention_coding_and_read_as_phrases();
}

fn header_advice_bullets_do_not_mention_coding_and_read_as_phrases() {
    let header = malvin::prompts::default_file("header.md").expect("header");
    let advice = header
        .split("you can find advice on")
        .nth(1)
        .expect("advice intro");
    let bullets: Vec<&str> = advice
        .lines()
        .skip(1)
        .map(str::trim)
        .take_while(|line| line.starts_with("- "))
        .map(|line| line.trim_start_matches("- ").trim())
        .collect();
    assert!(!bullets.is_empty(), "header should list advice topics");
    for bullet in bullets {
        for word in bullet.split(|c: char| !c.is_ascii_alphanumeric()) {
            assert!(
                !word.eq_ignore_ascii_case("code") && !word.eq_ignore_ascii_case("coding"),
                "VISION.md forbids header.md from mentioning coding; bullet {bullet:?}"
            );
        }
        assert!(
            !bullet.contains("various problem ("),
            "advice bullet is not a readable phrase: {bullet:?}"
        );
    }
}
