use std::path::{Path, PathBuf};

use crate::agent_process::AuthError;

pub fn ensure_claude_authenticated() -> Result<(), AuthError> {
    if has_claude_login() {
        return Ok(());
    }
    Err(AuthError(
        "Claude backend is not authenticated. Set ANTHROPIC_API_KEY, ANTHROPIC_OAUTH_TOKEN, ANTHROPIC_AUTH_TOKEN, or CLAUDE_CODE_OAUTH_TOKEN; alternatively, run `claude login`.".into(),
    ))
}

fn has_claude_login() -> bool {
    crate::agent_process::env_key_nonempty("ANTHROPIC_API_KEY")
        || crate::agent_process::env_key_nonempty("ANTHROPIC_OAUTH_TOKEN")
        || crate::agent_process::env_key_nonempty("ANTHROPIC_AUTH_TOKEN")
        || crate::agent_process::env_key_nonempty("CLAUDE_CODE_OAUTH_TOKEN")
        || claude_login_candidates(&crate::workspace::user_home::user_home_dir())
            .iter()
            .any(|path| path.is_file())
}

pub(crate) fn claude_login_candidates(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".claude.json"),
        home.join(".claude/.credentials.json"),
    ]
}
