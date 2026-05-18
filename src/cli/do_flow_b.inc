
async fn run_do_acp(
    client: &mut AgentClient,
    artifacts: &RunArtifacts,
    coder: DoCoderRun,
) -> Result<(), String> {
    let timing = client.attach_run_timing_for_session();
    if let Err(e) = client.begin_coder_session(&artifacts.work_dir).await {
        client.set_run_timing(None);
        return Err(e.to_string());
    }
    stamp_do_timing_display_name(&timing);
    let log = artifacts.log_path("do");
    let (ref header, ref user) = coder.header_user_for_trace;
    let do_split = Some((header.as_str(), user.as_str()));
    let run_res = client
        .run_coder_prompt(
            &coder.combined,
            &log,
            "do",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                skip_repo_style: coder.skip_repo_style,
                do_trace_split: do_split,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string());
    let end_res = client.end_coder_session().await.map_err(|e| e.to_string());
    let merged = merge_two_string_results(run_res, end_res);
    timing_merge::emit_run_timing_json_only_after_acp(client, &artifacts.run_dir, &timing, merged)
}

#[test]
fn stringify_do_flow_helpers() {
    let _ = stringify!(crate::cli::do_flow::prepare_do_prompt_store);
    let _ = stringify!(crate::cli::do_flow::run_do);
    let _ = stringify!(crate::cli::DoArgs);
    let _ = stringify!(crate::cli::do_flow::combine_do_acp_prompt_header_and_user);
    let _ = stringify!(crate::cli::do_flow::combine_do_raw_header_and_user);
    let _ = stringify!(crate::cli::do_flow::prepare_do_raw_prompt_store);
    let _ = stringify!(crate::cli::do_flow::DoCoderRun);
}

#[cfg(test)]
mod do_tests {
    use clap::Parser;

    use malvin::artifacts::RunArtifacts;
    use malvin::prompts::{DO_HEADER_MD, HEADER_MD, PromptStore};

    use super::{combine_do_acp_prompt_header_and_user, combine_do_raw_header_and_user};

    #[test]
    fn combine_do_acp_prompt_joins_rendered_header_and_request() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prompt_root = tmp.path().join("prompts");
        std::fs::create_dir_all(&prompt_root).expect("mkdir");
        std::fs::write(prompt_root.join(HEADER_MD), "HEADER_TOKEN\n").expect("header");
        std::fs::write(prompt_root.join("kpop_common.md"), "").expect("kpop_common");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ignored").expect("plan");
        let run_dir = tmp.path().join("_malvin").join("r");
        std::fs::create_dir_all(&run_dir).expect("run");
        let artifacts = RunArtifacts {
            run_dir,
            plan_path: plan,
            work_dir: tmp.path().to_path_buf(),
        };
        let store = PromptStore::with_root(prompt_root);
        let (combined, header, user) =
            combine_do_acp_prompt_header_and_user(&store, &artifacts, "USER_TOKEN")
                .expect("combine");
        assert_eq!(header, "HEADER_TOKEN");
        assert_eq!(user, "USER_TOKEN");
        assert_eq!(combined, "HEADER_TOKEN\n\nUSER_TOKEN");
        assert_eq!(combined.split("\n\n").count(), 2);
        assert_eq!(combined.matches("HEADER_TOKEN").count(), 1);
        assert_eq!(combined.matches("USER_TOKEN").count(), 1);
    }

    #[test]
    fn combine_do_raw_header_and_user_joins_rendered_do_header_and_request() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let prompt_root = tmp.path().join("prompts");
        std::fs::create_dir_all(&prompt_root).expect("mkdir");
        std::fs::write(prompt_root.join(DO_HEADER_MD), "DO_TOKEN\n").expect("do_header");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "ignored").expect("plan");
        let run_dir = tmp.path().join("_malvin").join("r");
        std::fs::create_dir_all(&run_dir).expect("run");
        let artifacts = RunArtifacts {
            run_dir,
            plan_path: plan,
            work_dir: tmp.path().to_path_buf(),
        };
        let store = PromptStore::with_root(prompt_root);
        let (combined, header, user) =
            combine_do_raw_header_and_user(&store, &artifacts, "USER_RAW_TOKEN\n\n")
                .expect("combine");
        assert_eq!(header, "DO_TOKEN");
        assert_eq!(user, "USER_RAW_TOKEN");
        assert_eq!(combined, "DO_TOKEN\n\nUSER_RAW_TOKEN");
        assert_eq!(combined.split("\n\n").count(), 2);
        assert_eq!(combined.matches("DO_TOKEN").count(), 1);
        assert_eq!(combined.matches("USER_RAW_TOKEN").count(), 1);
    }

    #[test]
    fn cli_accepts_do_and_passes_request() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "fix the bug"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert_eq!(d.request, "fix the bug");
                assert!(!d.cooked);
                assert!(!d.repo_gates);
                assert!(!d.thoughts);
            }
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_accepts_do_cooked() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "--cooked", "x"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert!(d.cooked);
                assert_eq!(d.request, "x");
                assert!(!d.repo_gates);
                assert!(!d.thoughts);
            }
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_accepts_do_repo_gates() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "--repo-gates", "y"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert!(d.repo_gates);
                assert_eq!(d.request, "y");
                assert!(!d.thoughts);
            }
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_accepts_do_thoughts() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "--thoughts", "z"]).expect("parse");
        match cli.command {
            Commands::Do(d) => {
                assert!(d.thoughts);
                assert_eq!(d.request, "z");
            }
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_accepts_all_shared_flags_before_subcommand() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from([
            "malvin",
            "--model",
            "composer-2",
            "--no-force",
            "--no-tee",
            "do",
            "z",
        ])
        .expect("parse");
        assert_eq!(cli.shared.model, "composer-2");
        assert!(cli.shared.no_tee);
        assert!(cli.shared.no_force);
        match cli.command {
            Commands::Do(d) => assert_eq!(d.request, "z"),
            _ => panic!("expected Do subcommand"),
        }
    }

    #[test]
    fn cli_defaults_to_verbose_and_accepts_verbose_flags() {
        use crate::cli::Cli;
        use crate::cli::Commands;

        let cli = Cli::try_parse_from(["malvin", "do", "default"]).expect("parse");
        assert!(cli.shared.verbose);
        match &cli.command {
            Commands::Do(d) => assert_eq!(d.request, "default"),
            _ => panic!("expected Do subcommand"),
        }

        let cli = Cli::try_parse_from(["malvin", "-v", "do", "x"]).expect("parse");
        assert!(cli.shared.verbose);
        match &cli.command {
            Commands::Do(d) => assert_eq!(d.request, "x"),
            _ => panic!("expected Do subcommand"),
        }

        let cli = Cli::try_parse_from(["malvin", "do", "--verbose", "y"]).expect("parse");
        assert!(cli.shared.verbose);
        match cli.command {
            Commands::Do(d) => assert_eq!(d.request, "y"),
            _ => panic!("expected Do subcommand"),
        }
    }
}
