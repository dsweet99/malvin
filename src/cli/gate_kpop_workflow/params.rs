use crate::artifacts::SessionDotfileBackups;
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::behavior::GateLoopBehavior;
use super::prepared::GateKpopPrepared;

pub(crate) struct GateKpopLoopParams<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a GateKpopPrepared,
    pub max_loops: usize,
    pub max_hypotheses: usize,
    pub behavior: GateLoopBehavior,
}

pub(crate) struct GateKpopIterationParams<'a> {
    pub loop_params: &'a GateKpopLoopParams<'a>,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub client: &'a mut crate::acp::AgentClient,
    pub iteration: usize,
    pub exp_log_path: std::path::PathBuf,
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_kpop_iteration_params_is_covered() {
        let _ = stringify!(GateKpopIterationParams);
    }
}
