use super::{RouterInitialPromptInput, build_router_initial_prompt};
use malvin::prompts::{HEADER_MD, KPOP_COMMON_MD, PromptStore, ROUTER_A_MD};
use malvin::test_utils::with_isolated_home;

fn write_minimal_router_prompts(prompt_root: &std::path::Path) {
    std::fs::create_dir_all(prompt_root).expect("mkdir");
    write_router_prompt_files(prompt_root);
}

fn write_router_prompt_files(prompt_root: &std::path::Path) {
    let files = [
        (HEADER_MD, "HEADER_BODY\n{{ kpop_insert }}\n"),
        (KPOP_COMMON_MD, "KPOP_BODY {{ max_hypotheses }}\n"),
        ("mbc2.md", "MBC2 {{ user_prompt }}\n"),
        (
            ROUTER_A_MD,
            "ROUTER_A {{ user_request_path }} {{ code_extra }}\n",
        ),
        ("router_code_extra.md", "CODE_EXTRA\n"),
        ("kpop_common_no_kpop.md", "\n"),
        ("router_a_no_kpop.md", "ROUTER_A_NO_KPOP {{ code_extra }}\n"),
    ];
    for (name, content) in files {
        std::fs::write(prompt_root.join(name), content).expect("write prompt");
    }
}

#[test]
fn initial_prompt_joins_header_kpop_and_router_a_in_order() {
    with_isolated_home(|work| {
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "req",
            Some(work),
            malvin::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_router_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let out = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: false,
            no_kpop: false,
            creative: false,
            max_hypotheses: 5,
            include_header: true,
            gate_iteration: 1,
        })
        .expect("initial");
        assert!(out.body.contains("HEADER_BODY"));
        assert!(out.body.contains("KPOP_BODY"));
        assert!(out.body.contains("ROUTER_A"));
        let header_at = out.body.find("HEADER_BODY").expect("header");
        let kpop_at = out.body.find("KPOP_BODY").expect("kpop");
        let a_at = out.body.find("ROUTER_A").expect("a");
        assert!(header_at < kpop_at && kpop_at < a_at);
        assert_eq!(
            out.stdout_label.split('+').collect::<Vec<_>>(),
            vec![HEADER_MD, ROUTER_A_MD]
        );
        assert_eq!(out.stdout_label, "header.md+router_a.md");
        assert_eq!(out.log_who, "router_initial");
        assert!(!out.body.contains("MBC2"));
    });
}

#[test]
fn initial_prompt_adds_mbc2_when_creative() {
    with_isolated_home(|work| {
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "creative request",
            Some(work),
            malvin::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_router_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let out = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: false,
            no_kpop: false,
            creative: true,
            max_hypotheses: 5,
            include_header: true,
            gate_iteration: 1,
        })
        .expect("initial");
        assert!(out.body.contains("MBC2"));
        assert!(out.stdout_label.contains("mbc2.md"));
        let kpop_at = out.body.find("KPOP_BODY").expect("kpop");
        let mbc2_at = out.body.find("MBC2").expect("mbc2");
        let a_at = out.body.find("ROUTER_A").expect("a");
        assert!(kpop_at < mbc2_at && mbc2_at < a_at);
    });
}

#[test]
fn initial_prompt_omits_header_when_not_included() {
    with_isolated_home(|work| {
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "req",
            Some(work),
            malvin::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_router_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let out = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: false,
            no_kpop: false,
            creative: false,
            max_hypotheses: 5,
            include_header: false,
            gate_iteration: 2,
        })
        .expect("initial");
        assert!(!out.body.contains("HEADER_BODY"));
        assert!(
            out.body.contains("KPOP_BODY"),
            "without header, kpop must still name this loop's exp log"
        );
        assert_eq!(
            out.stdout_label.split('+').collect::<Vec<_>>(),
            vec![KPOP_COMMON_MD, ROUTER_A_MD]
        );
    });
}

#[test]
fn initial_prompt_respects_no_kpop_and_gates() {
    with_isolated_home(|work| {
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "req",
            Some(work),
            malvin::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        malvin::seed_malvin_checks(artifacts.work_dir.as_path(), "echo INITIAL_GATE\n");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_router_prompts(&prompt_root);
        let store = PromptStore::with_root(prompt_root);
        let out = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: true,
            no_kpop: true,
            creative: false,
            max_hypotheses: 5,
            include_header: true,
            gate_iteration: 1,
        })
        .expect("initial");
        assert!(out.body.contains("HEADER_BODY"));
        assert!(!out.body.contains("KPOP_BODY"));
        assert!(out.body.contains("ROUTER_A_NO_KPOP"));
        assert!(out.body.contains("echo INITIAL_GATE") || out.body.contains("CODE_EXTRA"));
        assert!(
            !out.stdout_label
                .split('+')
                .any(|l| l == "kpop_common.md" || l == "kpop_common_no_kpop.md")
        );
        assert!(out.stdout_label.contains("router_a_no_kpop.md"));
    });
}

#[test]
fn initial_prompt_git_and_max_hypotheses_affect_composition() {
    with_isolated_home(|work| {
        let artifacts = malvin::artifacts::create_run_artifacts_from_text_opts(
            "req",
            Some(work),
            malvin::run_id::RunDirOptions::default(),
        )
        .expect("artifacts");
        let prompt_root = artifacts.run_dir.join("prompts");
        write_minimal_router_prompts(&prompt_root);
        std::fs::write(prompt_root.join(HEADER_MD), "HEADER\n{{ kpop_insert }}\n").expect("header");
        let store = PromptStore::with_root(prompt_root);
        let hi = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: false,
            no_kpop: false,
            creative: false,
            max_hypotheses: 7,
            include_header: true,
            gate_iteration: 1,
        })
        .expect("hi");
        let lo = build_router_initial_prompt(RouterInitialPromptInput {
            store: &store,
            artifacts: &artifacts,
            model: "cursor:auto",
            gates: false,
            no_kpop: false,
            creative: false,
            max_hypotheses: 3,
            include_header: true,
            gate_iteration: 1,
        })
        .expect("lo");
        assert_ne!(hi.body, lo.body);
        assert!(hi.body.contains("KPOP_BODY 7") || hi.body.contains('7'));
        assert!(lo.body.contains("KPOP_BODY 3") || lo.body.contains('3'));
        assert!(!hi.body.contains("write_a"));
        assert!(!hi.body.contains("write_b"));
    });
}
