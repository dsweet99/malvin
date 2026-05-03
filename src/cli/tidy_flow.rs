include!("tidy_flow/helpers.rs");

use clap::Args;
use malvin::output::{MALVIN_WHO, print_stdout_line};

#[derive(Args, Debug, Clone)]
pub struct TidyArgs {
    #[arg(long, default_value_t = false)]
    pub no_learn: bool,
}

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let (mut client, artifacts, kissconfig_backup, store, context, run_learn) =
        prepare_tidy_run(shared, workflow, !tidy.no_learn)?;
    let prompt = compose_tidy_prompt(&store, &context)?;
    let mut input = TidyAcpInput {
        client: &mut client,
        artifacts: &artifacts,
        store: &store,
        context: &context,
        run_learn,
    };
    let result = run_tidy_acp(&mut input, prompt.trim_end(), &kissconfig_backup).await;
    merge_tidy_timing(result, &artifacts, &kissconfig_backup)?;
    print_stdout_line(MALVIN_WHO, "DONE");
    Ok(())
}

mod coverage_tests;
