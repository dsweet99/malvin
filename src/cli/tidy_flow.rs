include!("tidy_flow/helpers.rs");

use clap::Args;
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    /// Re-run the tidy.md coder turn up to this many times while workspace gates still fail after each tidy (`kiss check` etc.), before learn and summary.
    #[arg(long, default_value_t = 1)]
    pub max_loops: usize,
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (
        mut client,
        artifacts,
        session_dotfile_backups,
        store,
        context,
        run_learn,
    ) = prepare_tidy_run(shared, workflow, !tidy.no_learn)?;
    super::error_run_log::set_command_error_run_dir(Some(artifacts.run_dir.clone()));
    let r = async {
        if tidy.max_loops == 0 {
            return Err("max-loops must be at least 1".to_string());
        }
        let prompt = compose_tidy_prompt(&store, &context)?;
        let mut input = TidyAcpInput {
            client: &mut client,
            artifacts: &artifacts,
            store: &store,
            context: &context,
            run_learn,
        };
        let result = run_tidy_acp(
            &mut input,
            prompt.trim_end(),
            &session_dotfile_backups,
            tidy.max_loops,
        )
        .await;
        merge_tidy_timing(
            result,
            &artifacts,
            &session_dotfile_backups,
        )?;
        print_stdout_line(MALVIN_WHO, "DONE");
        Ok(())
    }
    .await;
    if r.is_ok() {
        super::error_run_log::clear_command_error_run_dir();
    }
    r
}

mod coverage_tests;
