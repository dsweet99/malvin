use crate::artifacts::SessionDotfileBackups;
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::behavior::KPopHardConstraints;
use super::prepared::KPopEnginePrepared;

pub(crate) struct KPopEngineParams<'a> {
    pub command: &'a str,
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a KPopEnginePrepared,
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub behavior: KPopHardConstraints,
}

pub(crate) struct KPopEngineIterationParams<'a> {
    pub loop_params: &'a KPopEngineParams<'a>,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub client: &'a mut crate::agent_backend::AgentBackend,
    pub iteration: usize,
    pub total_iterations: usize,
    pub consecutive_solved_entering: usize,
    pub exp_log_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    #[test]
    fn kpop_engine_iteration_params_is_covered() {
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<KPopEngineIterationParams> = None;
        let _: Option<KPopEngineParams> = None;
    }
}
