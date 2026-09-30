use crate::cli::one_shot_session::OneShotCoderGuard;
use malvin::agent_backend::SdkClient;
use malvin::artifacts::RunArtifacts;

use super::do_flow_prompt;

pub(super) async fn run_do_acp(
    client: &mut SdkClient,
    artifacts: &RunArtifacts,
    coder: do_flow_prompt::DoCoderRun,
) -> Result<(), String> {
    let _ = coder;
    let guard = OneShotCoderGuard::begin(client, artifacts, "do").await?;
    guard.finish(client, Ok(())).await
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::run_do_acp;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_do_acp;
    }
}
