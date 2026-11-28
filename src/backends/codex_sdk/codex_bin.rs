use std::path::PathBuf;

use crate::workspace::support_paths::{OverrideKind, env_path_override};

pub const CODEX_MISSING_HINT: &str = "codex backend requires the codex binary on PATH (or MALVIN_CODEX); install Codex CLI separately.";
pub const DEFAULT_CODEX_LIST_MODELS_TIMEOUT_MS: u64 = 30_000;

#[must_use]
pub fn codex_missing_binary_message() -> String {
    CODEX_MISSING_HINT.to_string()
}

pub fn resolve_codex_bin() -> Result<PathBuf, String> {
    if let Some(result) =
        env_path_override("MALVIN_CODEX", OverrideKind::Executable, CODEX_MISSING_HINT)
    {
        return result;
    }
    crate::workspace::support_paths::lookup_bin_on_path("codex")
        .ok_or_else(codex_missing_binary_message)
}
