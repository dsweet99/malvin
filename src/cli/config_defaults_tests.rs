use super::{
    apply_loop_defaults, apply_shared_config_defaults, apply_workspace_config_defaults,
    global_flag_from_command_line, parse_cli_with_config_defaults, LoopDefaultMut,
};
use crate::cli::config_loop::subcommand_flag_from_command_line;
use crate::cli::{Cli, Commands, SharedOpts};
use crate::malvin_config_file::{AgentConfig, DEFAULT_MAX_HYPOTHESES, DEFAULT_MAX_LOOPS};
use crate::support_paths::{DEFAULT_CLI_MODEL, DEFAULT_MAX_ACP_RETRIES};
use clap::{CommandFactory, FromArgMatches};

fn write_agent_config(work_dir: &std::path::Path) {
    let path = crate::malvin_config_path(work_dir);
    let mut text = std::fs::read_to_string(&path).expect("read");
    if text.contains("[agent]") {
        text = text
            .lines()
            .filter(|line| {
                !line.starts_with("model =")
                    && !line.starts_with("max_hypotheses =")
                    && !line.starts_with("max_loops =")
                    && !line.starts_with("max_loops_code =")
                    && !line.starts_with("max_acp_retries =")
                    && *line != "[agent]"
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    text.push_str(
        "\n[agent]\nmodel = \"cfg-model\"\nmax_hypotheses = 42\nmax_loops = 9\nmax_loops_code = 7\nmax_acp_retries = 8\n",
    );
    std::fs::write(&path, text).expect("write");
}

fn with_seeded_agent_config(f: impl FnOnce()) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    crate::malvin_config_file::open_malvin_config(tmp.path()).expect("seed");
    write_agent_config(tmp.path());
    f();
    std::env::set_current_dir(cwd).expect("restore cwd");
}

#[test]
fn write_agent_config_adds_agent_section_to_partial_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = crate::malvin_config_path(tmp.path());
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, "mem_limit_gb = 2\n").expect("write");
    write_agent_config(tmp.path());
    let text = std::fs::read_to_string(&path).expect("read");
    assert!(text.contains("[agent]"));
    assert!(text.contains("model = \"cfg-model\""));
}

#[test]
fn apply_loop_defaults_honors_partial_cli_overrides() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin", "kpop", "--max-loops", "3", "hello",
        ]);
        let mut max_loops = 3_usize;
        let mut max_hypotheses = 1_usize;
        apply_loop_defaults(
            &matches,
            "kpop",
            LoopDefaultMut {
                max_loops: &mut max_loops,
                max_hypotheses: &mut max_hypotheses,
                config_max_loops: 9,
                config_max_hypotheses: 42,
            },
        );
        assert_eq!((max_loops, max_hypotheses), (3, 42));
    });
}

#[test]
fn flag_and_shared_helpers_detect_and_apply_defaults() {
    let matches = Cli::command().get_matches_from(["malvin", "kpop", "hello"]);
    assert!(!global_flag_from_command_line(&matches, "model"));
    assert!(!subcommand_flag_from_command_line(&matches, "kpop", "max_loops"));
    assert!(!subcommand_flag_from_command_line(&matches, "missing", "max_loops"));

    let agent = AgentConfig {
        model: "cfg".into(),
        max_hypotheses: 40,
        max_loops: 8,
        max_loops_code: 6,
        max_acp_retries: 6,
    };
    let config_max_hypotheses = agent.max_hypotheses;
    let config_max_loops = agent.max_loops;
    let mut shared = SharedOpts {
        model: "old".into(),
        no_force: false,
        no_tee: false,
        no_markdown: false,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
    };
    apply_shared_config_defaults(&matches, &mut shared, &agent);
    assert_eq!(shared.model, "cfg");
    assert_eq!(shared.max_acp_retries, 6);

    let mut max_loops = 1_usize;
    let mut max_hypotheses = 1_usize;
    apply_loop_defaults(
        &matches,
        "kpop",
        LoopDefaultMut {
            max_loops: &mut max_loops,
            max_hypotheses: &mut max_hypotheses,
            config_max_loops,
            config_max_hypotheses,
        },
    );
    assert_eq!((max_loops, max_hypotheses), (8, 40));
}

#[test]
fn apply_workspace_config_defaults_overrides_unset_flags() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "kpop", "hello"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cfg-model");
        assert_eq!(cli.shared.max_acp_retries, 8);
        match cli.command.expect("command") {
            Commands::Kpop(kpop) => assert_eq!((kpop.max_loops, kpop.max_hypotheses), (9, 42)),
            other => panic!("expected kpop, got {other:?}"),
        }
    });
}

#[test]
fn apply_workspace_config_defaults_respects_explicit_cli_flags() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin", "--model", "cli-model", "--max-acp-retries", "2", "kpop",
            "--max-loops", "3", "--max-hypotheses", "4", "hello",
        ]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cli-model");
        assert_eq!(cli.shared.max_acp_retries, 2);
        match cli.command.expect("command") {
            Commands::Kpop(kpop) => assert_eq!((kpop.max_loops, kpop.max_hypotheses), (3, 4)),
            other => panic!("expected kpop, got {other:?}"),
        }
    });
}

fn assert_workflow_defaults(argv: &[&str]) {
    let matches = Cli::command().get_matches_from(argv);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
    match cli.command.expect("command") {
        Commands::Code(a) => assert_eq!((a.max_loops, a.max_hypotheses), (7, 42)),
        Commands::Constrain(a) => assert_eq!((a.max_loops, a.max_hypotheses), (9, 42)),
        Commands::Tidy(a) => assert_eq!((a.max_loops, a.max_hypotheses), (7, 42)),
        other => panic!("unexpected command {other:?}"),
    }
}

#[test]
fn apply_workspace_config_defaults_for_workflow_commands() {
    with_seeded_agent_config(|| {
        assert_workflow_defaults(&["malvin", "code", "hello"]);
        assert_workflow_defaults(&["malvin", "constrain", "hello"]);
        assert_workflow_defaults(&["malvin", "tidy"]);
    });
}

#[test]
fn apply_workspace_config_defaults_skips_do() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let config_path = crate::malvin_config_path(tmp.path());
    assert!(!config_path.exists());
    let do_matches = Cli::command().get_matches_from(["malvin", "do", "hello"]);
    let mut do_cli = Cli::from_arg_matches(&do_matches).expect("cli");
    apply_workspace_config_defaults(&do_matches, &mut do_cli).expect("apply");
    assert!(!config_path.exists());
    std::env::set_current_dir(cwd).expect("restore cwd");
}

#[test]
fn apply_workspace_config_defaults_for_invent() {
    with_seeded_agent_config(|| {
        let invent = Cli::command().get_matches_from(["malvin", "invent", "ideas"]);
        let mut invent_cli = Cli::from_arg_matches(&invent).expect("cli");
        apply_workspace_config_defaults(&invent, &mut invent_cli).expect("apply");
        assert_eq!(invent_cli.shared.model, "cfg-model");
    });
}

#[test]
fn parse_cli_with_config_defaults_bare_request_resolves_to_kpop() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let cli = parse_cli_with_config_defaults(["malvin", "hello"]).expect("parse");
    match cli.command.expect("command") {
        Commands::Kpop(kpop) => assert_eq!(kpop.request.as_deref(), Some("hello")),
        other => panic!("expected kpop, got {other:?}"),
    }
    std::env::set_current_dir(cwd).expect("restore cwd");
}

#[test]
fn parse_cli_with_config_defaults_uses_bundled_agent_defaults() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let cwd = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let cli = parse_cli_with_config_defaults(["malvin", "kpop", "hello"]).expect("parse");
    assert_eq!(cli.shared.model, DEFAULT_CLI_MODEL);
    assert_eq!(cli.shared.max_acp_retries, DEFAULT_MAX_ACP_RETRIES);
    match cli.command.expect("command") {
        Commands::Kpop(kpop) => {
            assert_eq!(kpop.max_loops, DEFAULT_MAX_LOOPS);
            assert_eq!(kpop.max_hypotheses, DEFAULT_MAX_HYPOTHESES);
        }
        other => panic!("expected kpop, got {other:?}"),
    }
    std::env::set_current_dir(cwd).expect("restore cwd");
}
