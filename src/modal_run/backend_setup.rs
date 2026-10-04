use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::model_id::{ModelBackend, parse_model_id};

use super::backends::{PI_DIR, PI_ENTRY, PI_PACKAGE, local_codex_version, local_pi_version};
use super::credentials::{env_set, modal_credentials_present};

pub trait ModalBackendSetup: Sync {
    fn image_layer(&self, home: &Path) -> Option<Vec<String>>;
    fn sandbox_env(&self) -> BTreeMap<String, String>;
    fn login_candidates(&self, home: &Path) -> Vec<PathBuf>;
    fn credential_preflight(&self, home: &Path) -> Result<(), String>;

    fn login_files(&self, home: &Path) -> Vec<PathBuf> {
        self.login_candidates(home)
            .into_iter()
            .filter(|p| p.is_file())
            .collect()
    }
}

struct CursorSetup;
struct PiSetup;
struct CodexSetup;

#[must_use]
pub const fn modal_backend_setup(backend: ModelBackend) -> &'static dyn ModalBackendSetup {
    match backend {
        ModelBackend::Cursor => &CursorSetup,
        ModelBackend::Pi => &PiSetup,
        ModelBackend::Codex => &CodexSetup,
    }
}

pub fn setup_for(model: &str) -> Result<&'static dyn ModalBackendSetup, String> {
    parse_model_id(model).map(|m| modal_backend_setup(m.backend))
}

pub fn preflight(model: &str, home: &Path) -> Result<(), String> {
    if !modal_credentials_present(home) {
        return Err(
            "malvin --remote=modal needs Modal credentials. Run `modal setup`, or set MODAL_TOKEN_ID and MODAL_TOKEN_SECRET."
                .to_string(),
        );
    }
    setup_for(model)?.credential_preflight(home)
}

impl ModalBackendSetup for CursorSetup {
    fn image_layer(&self, _home: &Path) -> Option<Vec<String>> {
        None
    }

    fn sandbox_env(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn login_candidates(&self, _home: &Path) -> Vec<PathBuf> {
        Vec::new()
    }

    fn credential_preflight(&self, _home: &Path) -> Result<(), String> {
        if crate::cursor_sdk::effective_sdk_api_key().is_some() {
            return Ok(());
        }
        Err(
            "malvin --remote=modal with a cursor: model needs CURSOR_API_KEY (or CURSOR_AGENT_API_KEY / AGENT_API_KEY); `agent login` does not reach the Sandbox."
                .to_string(),
        )
    }
}

impl ModalBackendSetup for PiSetup {
    fn image_layer(&self, home: &Path) -> Option<Vec<String>> {
        Some(vec![format!(
            "RUN mkdir -p {PI_DIR} && cd {PI_DIR} && npm install --no-audit --no-fund {PI_PACKAGE}@{}",
            local_pi_version(home)
        )])
    }

    fn sandbox_env(&self) -> BTreeMap<String, String> {
        BTreeMap::from([("MALVIN_PI".to_string(), PI_ENTRY.to_string())])
    }

    fn login_candidates(&self, home: &Path) -> Vec<PathBuf> {
        let dir = std::env::var_os("PI_CODING_AGENT_DIR")
            .filter(|v| !v.is_empty())
            .map_or_else(|| home.join(".pi/agent"), PathBuf::from);
        vec![dir.join("auth.json"), dir.join("models.json")]
    }

    fn credential_preflight(&self, _home: &Path) -> Result<(), String> {
        Ok(())
    }
}

impl ModalBackendSetup for CodexSetup {
    fn image_layer(&self, _home: &Path) -> Option<Vec<String>> {
        Some(vec![format!(
            "RUN npm install -g --no-audit --no-fund @openai/codex@{}",
            local_codex_version()
        )])
    }

    fn sandbox_env(&self) -> BTreeMap<String, String> {
        BTreeMap::new()
    }

    fn login_candidates(&self, home: &Path) -> Vec<PathBuf> {
        vec![home.join(".codex/auth.json")]
    }

    fn credential_preflight(&self, home: &Path) -> Result<(), String> {
        if env_set("OPENAI_API_KEY") || !self.login_files(home).is_empty() {
            return Ok(());
        }
        Err("malvin --remote=modal with a codex: model needs OPENAI_API_KEY or ~/.codex/auth.json.".to_string())
    }
}
