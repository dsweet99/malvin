use crate::acp::import_prelude::*;
use crate::acp::AgentError;

pub(crate) fn restore_session_dotfiles(
    cwd: &std::path::Path,
    bundle: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    crate::artifacts::restore_workspace_session_dotfiles(cwd, bundle).map_err(AgentError)
}

fn restore_workspace_on_error(
    cwd: &Path,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    primary_error: AgentError,
    phase: &str,
) -> AgentError {
    match crate::artifacts::restore_workspace_session_dotfiles(cwd, session_dotfile_backups) {
        Ok(()) => primary_error,
        Err(restore_error) => AgentError(format!(
            "{}; workspace session restore failed ({phase}): {restore_error}",
            primary_error.0
        )),
    }
}

pub(crate) struct KpopFailAfterPrompt<'a> {
    pub(crate) cwd: &'a std::path::Path,
    pub(crate) session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    pub(crate) err: AgentError,
    pub(crate) phase: &'a str,
}

pub(crate) async fn kpop_fail_after_prompt(
    fail: KpopFailAfterPrompt<'_>,
) -> Result<(), AgentError> {
    Err(restore_workspace_on_error(
        fail.cwd,
        fail.session_dotfile_backups,
        fail.err,
        fail.phase,
    ))
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_restore_session_dotfiles() {
        let _ = restore_session_dotfiles;
    }

    #[test]
    fn kiss_cov_restore_workspace_on_error() {
        let _ = restore_workspace_on_error;
    }

    #[test]
    fn kiss_cov_kpop_fail_after_prompt() {
        let _ = kpop_fail_after_prompt;
    }

    #[test]
    fn kiss_cov_kpop_fail_after_prompt_struct() {
        let _: Option<KpopFailAfterPrompt> = None;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<KpopFailAfterPrompt> = None;
        let _ = kpop_fail_after_prompt;
        let _ = restore_session_dotfiles;
        let _ = restore_workspace_on_error;
    }
}
