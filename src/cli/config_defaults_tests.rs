use super::{
    apply_loop_defaults, apply_shared_config_defaults, apply_workspace_config_defaults,
    global_flag_from_command_line, parse_cli_with_config_defaults, LoopDefaultMut,
};
use crate::cli::config_loop::subcommand_flag_from_command_line;
use crate::cli::{Cli, Commands, SharedOpts};
use crate::malvin_config_file::AgentConfig;
use clap::{CommandFactory, FromArgMatches};

pub(super) fn write_agent_config(work_dir: &std::path::Path) {
    let path = crate::malvin_config_path(work_dir);
    let mut text = std::fs::read_to_string(&path).expect("read");
    if text.contains("[agent]") {
        text = text
            .lines()
            .filter(|line| {
                !line.starts_with("model =")
                    && !line.starts_with("max_loops =")
                    && !line.starts_with("max_loops_code =")
                    && !line.starts_with("max_acp_retries =")
                    && *line != "[agent]"
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    text.push_str(
        "\n[agent]\nmodel = \"cursor:cfg-model\"\nmax_loops = 9\nmax_loops_code = 7\nmax_acp_retries = 8\n",
    );
    std::fs::write(&path, text).expect("write");
}

pub(super) fn with_seeded_agent_config(f: impl FnOnce()) {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        crate::malvin_config_file::open_malvin_config(work).expect("seed");
        write_agent_config(work);
        f();
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}

#[test]
fn write_agent_config_adds_agent_section_to_partial_file() {
    crate::test_utils::with_isolated_home(|work| {
        let path = crate::malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 2\n").expect("write");
        write_agent_config(work);
        let text = std::fs::read_to_string(&path).expect("read");
        assert!(text.contains("[agent]"));
        assert!(text.contains("model = \"cursor:cfg-model\""));
    });
}

#[test]
fn apply_loop_defaults_honors_partial_cli_overrides() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin", "tidy", "--max-loops", "3",
        ]);
        let mut max_loops = 3_usize;
        let mut max_hypotheses = 5_usize;
        apply_loop_defaults(
            &matches,
            "tidy",
            LoopDefaultMut {
                max_loops: &mut max_loops,
                max_hypotheses: &mut max_hypotheses,
                config_max_loops: 9,
                config_max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
            },
        );
        assert_eq!(max_loops, 3);
    });
}

#[test]
fn flag_and_shared_helpers_detect_and_apply_defaults() {
    let matches = Cli::command().get_matches_from(["malvin", "tidy"]);
    assert!(!global_flag_from_command_line(&matches, "model"));
    assert!(!subcommand_flag_from_command_line(&matches, "tidy", "max_loops"));
    assert!(!subcommand_flag_from_command_line(&matches, "missing", "max_loops"));

    let agent = AgentConfig {
        model: "cursor:cfg".into(),
        max_loops: 8,
        max_hypotheses: crate::malvin_config_file::DEFAULT_MAX_HYPOTHESES,
        max_loops_code: 6,
        max_acp_retries: 6,
        max_mini_transport_retries: 3,
    };
    let config_max_loops = agent.max_loops;
    let mut shared = SharedOpts {
        model: "old".into(),
        no_force: false,
        no_tenacious: false,
        gates: false,

        quiet: false,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 32,
        mini_max_bash_execs: 128,
        mini_max_http_retries: 0,
        mini_max_transport_retries: crate::support_paths::DEFAULT_MAX_MINI_TRANSPORT_RETRIES,
        mini_max_gate_retries: 0,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    };
    apply_shared_config_defaults(&matches, &mut shared, &agent);
    assert_eq!(shared.model, "cursor:cfg");
    assert_eq!(shared.max_acp_retries, 6);

    let mut max_loops = 1_usize;
    let mut max_hypotheses = 1_usize;
    apply_loop_defaults(
        &matches,
        "tidy",
        LoopDefaultMut {
            max_loops: &mut max_loops,
            max_hypotheses: &mut max_hypotheses,
            config_max_loops,
            config_max_hypotheses: agent.max_hypotheses,
        },
    );
    assert_eq!(max_loops, 8);
}

#[test]
fn apply_workspace_config_defaults_overrides_unset_flags() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from(["malvin", "tidy"]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cursor:cfg-model");
        assert_eq!(cli.shared.max_acp_retries, 8);
        match cli.command.expect("command") {
            Commands::Tidy(tidy) => assert_eq!(tidy.max_loops, 7),
            other => panic!("expected tidy, got {other:?}"),
        }
    });
}

#[test]
fn apply_workspace_config_defaults_respects_explicit_cli_flags() {
    with_seeded_agent_config(|| {
        let matches = Cli::command().get_matches_from([
            "malvin", "--model", "cursor:cli-model", "--max-acp-retries", "2", "tidy",
            "--max-loops", "3",
        ]);
        let mut cli = Cli::from_arg_matches(&matches).expect("cli");
        apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
        assert_eq!(cli.shared.model, "cursor:cli-model");
        assert_eq!(cli.shared.max_acp_retries, 2);
        match cli.command.expect("command") {
            Commands::Tidy(tidy) => assert_eq!(tidy.max_loops, 3),
            other => panic!("expected tidy, got {other:?}"),
        }
    });
}

fn assert_workflow_defaults(argv: &[&str]) {
    let matches = Cli::command().get_matches_from(argv);
    let mut cli = Cli::from_arg_matches(&matches).expect("cli");
    apply_workspace_config_defaults(&matches, &mut cli).expect("apply");
    match cli.command.expect("command") {
        Commands::Tidy(a) => assert_eq!(a.max_loops, 7),
        Commands::Write(a) => assert_eq!(a.max_loops, 7),
        other => panic!("unexpected command {other:?}"),
    }
}

#[test]
fn apply_workspace_config_defaults_for_workflow_commands() {
    with_seeded_agent_config(|| {
        assert_workflow_defaults(&["malvin", "tidy"]);
        assert_workflow_defaults(&["malvin", "write", "topic"]);
    });
}

#[test]
fn apply_workspace_config_defaults_skips_do() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let config_path = crate::malvin_config_path(work);
        assert!(!config_path.exists());
        let do_matches = Cli::command().get_matches_from(["malvin", "--do", "hello"]);
        let mut do_cli = Cli::from_arg_matches(&do_matches).expect("cli");
        apply_workspace_config_defaults(&do_matches, &mut do_cli).expect("apply");
        assert!(!config_path.exists());
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}

#[test]
fn apply_workspace_config_defaults_for_inspire() {
    with_seeded_agent_config(|| {
        let inspire = Cli::command().get_matches_from(["malvin", "inspire", "ideas"]);
        let mut inspire_cli = Cli::from_arg_matches(&inspire).expect("cli");
        apply_workspace_config_defaults(&inspire, &mut inspire_cli).expect("apply");
        assert_eq!(inspire_cli.shared.model, "cursor:cfg-model");
    });
}

#[test]
fn parse_cli_with_config_defaults_tidy() {
    crate::test_utils::with_isolated_home(|work| {
        let cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(work).expect("chdir");
        let (cli, _) = parse_cli_with_config_defaults(["malvin", "tidy"]).expect("parse");
        match cli.command.expect("command") {
            Commands::Tidy(t) => {
                assert!(t.max_loops >= 1);
            }
            other => panic!("expected tidy, got {other:?}"),
        }
        std::env::set_current_dir(cwd).expect("restore cwd");
    });
}
