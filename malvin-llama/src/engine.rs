//! Local engine load / complete entry points.

use std::path::Path;
use std::sync::Mutex;

use crate::chat::ChatTurn;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
use crate::engine_metal as platform;

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
use crate::engine_stub as platform;

/// Loaded GGUF model ready for chat completions.
pub struct LocalEngine {
    inner: Mutex<platform::InnerEngine>,
}

/// Arguments for a local chat completion.
pub struct CompleteRequest<'a> {
    pub turns: &'a [ChatTurn],
    pub max_tokens: i32,
}

/// Load a GGUF model from disk (Metal Apple Silicon only in v1).
///
/// # Errors
///
/// Returns an error when the platform is unsupported, the path is missing, or load fails.
pub fn load_engine(gguf_path: &Path) -> Result<LocalEngine, String> {
    // llama-cpp-2 debug-asserts that the path exists; check first so callers get Err.
    if !gguf_path.is_file() {
        return Err(format!("GGUF not found: {}", gguf_path.display()));
    }
    let inner = platform::InnerEngine::load(gguf_path)?;
    Ok(LocalEngine {
        inner: Mutex::new(inner),
    })
}

/// Generate an assistant reply for `request.turns` (full conversation each call).
///
/// # Errors
///
/// Returns an error when generation fails.
pub fn complete(engine: &LocalEngine, request: &CompleteRequest<'_>) -> Result<String, String> {
    let guard = engine
        .inner
        .lock()
        .map_err(|_| "local llama engine lock poisoned".to_string())?;
    guard.complete(request)
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod engine_tests;
