//! Prime + malvin GGUF: OpenAI-compatible sidecar + temp `models.json`.

use std::path::PathBuf;

use tempfile::TempDir;

use super::openai_server::LocalOpenAiServer;
use super::prime_models_json::write_prime_local_models_json;
use super::{ensure_local_engine, DownloadPolicy};

/// Owns the localhost completion server and registry file for one Prime session.
pub struct PrimeLocalSidecar {
    _server: LocalOpenAiServer,
    _dir: TempDir,
    pub models_json_path: PathBuf,
}

impl PrimeLocalSidecar {
    /// Load catalog GGUF, serve it, and write Prime `models.json` for `local/<slug>`.
    ///
    /// # Errors
    ///
    /// Returns an error when the model cannot load, the server cannot bind, or the
    /// registry file cannot be written.
    pub fn start(model_id: &str, allow_download: bool) -> Result<Self, String> {
        let policy = if allow_download {
            DownloadPolicy::Allow
        } else {
            DownloadPolicy::Deny
        };
        let engine = ensure_local_engine(model_id, policy)?;
        let slug = engine.model_slug.clone();
        let server = LocalOpenAiServer::start(engine)?;
        let dir = TempDir::new().map_err(|e| format!("prime local tempdir: {e}"))?;
        let models_json_path = dir.path().join("models.json");
        let display = crate::local_llm::require_known_local_slug(&slug)?.display_name;
        write_prime_local_models_json(
            &models_json_path,
            &server.base_url,
            &slug,
            display,
        )?;
        Ok(Self {
            _server: server,
            _dir: dir,
            models_json_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_fails_fast_on_unknown_slug() {
        let err = PrimeLocalSidecar::start("prime:local/not_a_model", false)
            .err()
            .expect("err");
        assert!(err.contains("unknown local model"), "{err}");
        let _ = stringify!(models_json_path);
        let _ = stringify!(PrimeLocalSidecar);
    }
}
