//! `malvin models` — list Cursor, `OpenRouter`, and `local:` models with prefixes.

use std::path::PathBuf;

use crate::agent_or_cursor_agent_bin;
use crate::ansi_strip::strip_ansi_escapes;
use crate::local_llm::{download_local_model, local_model_listings};
use crate::model_id::{CURSOR_PREFIX, LOCAL_PREFIX, OPENROUTER_PREFIX};
use crate::output::{MALVIN_WHO, print_stdout_line};
use clap::Args;

#[path = "models_cmd_parse.rs"]
mod models_cmd_parse;
use models_cmd_parse::{print_parsed_or_fallback_prefixed, trim_trailing_tip_lines};

#[derive(Args, Debug, Clone, Default)]
pub struct ModelsArgs {
    /// Optional words: `download local:<id>` fetches a model into `~/.malvin_home/model_cache`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub words: Vec<String>,
}

#[cfg(test)]
pub(crate) const fn models_args_marker(_args: &ModelsArgs) -> &'static str {
    "models"
}

fn resolve_models_cli() -> Result<PathBuf, String> {
    agent_or_cursor_agent_bin().ok_or_else(|| {
        "Neither `agent` nor `cursor-agent` was found on PATH. Install the Cursor CLI agent to use `malvin models`."
            .to_string()
    })
}

fn print_current_footer(current_model: &str) {
    print_stdout_line(MALVIN_WHO, "");
    print_stdout_line(MALVIN_WHO, &format!("Current: {current_model}"));
}

/// Print Cursor, `OpenRouter`, and `local:` models with prefixes and a `Current:` footer.
///
/// When `words` is `download <local:id>`, downloads that model into the cache instead.
pub fn run_models(args: ModelsArgs, current_model: &str) -> Result<(), String> {
    if !args.words.is_empty() {
        return run_models_action(&args.words);
    }
    print_cursor_models()?;
    // OpenRouter listing is best-effort when the API key / network is unavailable.
    match list_openrouter_models_sync() {
        Ok(models) => print_openrouter_models(&models),
        Err(e) => {
            print_stdout_line(MALVIN_WHO, &format!("(openrouter models unavailable: {e})"));
        }
    }
    print_local_models();
    print_current_footer(current_model);
    Ok(())
}

fn run_models_action(words: &[String]) -> Result<(), String> {
    match words {
        [action, model] if action == "download" => {
            let path = download_local_model(model)?;
            print_stdout_line(
                MALVIN_WHO,
                &format!("downloaded {model} -> {}", path.display()),
            );
            Ok(())
        }
        [action] if action == "download" => Err(
            "usage: malvin models download local:<id> (e.g. local:qwen35_9b_q4)".into(),
        ),
        _ => Err(format!(
            "unknown models action {words:?}; try `malvin models` or `malvin models download local:<id>`"
        )),
    }
}

fn list_openrouter_models_sync() -> Result<Vec<malvin_mini::ModelListing>, String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create Tokio runtime: {e}"))?;
    rt.block_on(async {
        use malvin_mini::{OpenRouterClient, OpenRouterConfig};
        let config = OpenRouterConfig::from_env_for_listing()?;
        let client = OpenRouterClient::new(config).map_err(|e| e.to_string())?;
        client.list_models().await.map_err(|e| e.to_string())
    })
}

/// Fetch `OpenRouter` models (async helper for tests).
#[cfg(any(test, doctest))]
pub async fn run_mini_models() -> Result<(), String> {
    use malvin_mini::{OpenRouterClient, OpenRouterConfig};

    let config = OpenRouterConfig::from_env_for_listing()?;
    let client = OpenRouterClient::new(config).map_err(|e| e.to_string())?;
    let models = client.list_models().await.map_err(|e| e.to_string())?;
    print_openrouter_models(&models);
    print_current_footer(crate::config::DEFAULT_CLI_MODEL);
    Ok(())
}

fn print_cursor_models() -> Result<(), String> {
    let bin = resolve_models_cli()?;
    let output = crate::malvin_sandbox::malvin_std_command(&bin)
        .arg("models")
        .output()
        .map_err(|e| format!("failed to execute {} models: {e}", bin.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        let detail = if msg.is_empty() {
            format!("`{} models` exited with {}", bin.display(), output.status)
        } else {
            format!("`{} models` failed: {msg}", bin.display())
        };
        return Err(detail);
    }

    let raw = String::from_utf8_lossy(&output.stdout);
    let text = strip_ansi_escapes(raw.as_ref());
    let cleaned = trim_trailing_tip_lines(&text);
    print_parsed_or_fallback_prefixed(&cleaned, CURSOR_PREFIX);
    Ok(())
}

fn print_openrouter_models(models: &[malvin_mini::ModelListing]) {
    for model in models {
        print_stdout_line(
            MALVIN_WHO,
            &format!("{OPENROUTER_PREFIX}{}\t{}", model.id, model.name),
        );
    }
}

fn print_local_models() {
    for model in local_model_listings() {
        print_stdout_line(
            MALVIN_WHO,
            &format!("{LOCAL_PREFIX}{}\t{}", model.id, model.name),
        );
    }
}

#[cfg(test)]
pub(crate) mod test_hooks {
    use super::models_cmd_parse;

    pub struct EnvGuard {
        key: &'static str,
        prior: Option<String>,
    }

    impl EnvGuard {
        #[allow(unsafe_code)]
        pub fn set(key: &'static str, value: Option<&str>) -> Self {
            let prior = std::env::var(key).ok();
            unsafe {
                match value {
                    Some(v) => std::env::set_var(key, v),
                    None => std::env::remove_var(key),
                }
            }
            Self { key, prior }
        }
    }

    impl Drop for EnvGuard {
        #[allow(unsafe_code)]
        fn drop(&mut self) {
            unsafe {
                match &self.prior {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }

    pub fn trim_trailing_tip_lines(text: &str) -> String {
        models_cmd_parse::trim_trailing_tip_lines(text)
    }

    pub fn looks_like_tip_banner_line(lowercase_trimmed: &str) -> bool {
        models_cmd_parse::looks_like_tip_banner_line(lowercase_trimmed)
    }

    pub fn is_models_section_header(line: &str) -> bool {
        models_cmd_parse::is_non_model_banner_line(line)
    }

    pub fn models_display_lines(text: &str) -> Option<Vec<String>> {
        models_cmd_parse::models_display_lines(text, "")
    }

    pub fn print_parsed_or_fallback(text: &str) {
        models_cmd_parse::print_parsed_or_fallback_prefixed(text, "");
    }

    pub fn parse_model_line(line: &str) -> Option<(&str, String)> {
        models_cmd_parse::parse_model_line(line)
    }

    pub fn resolve_models_cli() -> Result<std::path::PathBuf, String> {
        super::resolve_models_cli()
    }

    pub fn print_mini_models(models: &[malvin_mini::ModelListing]) {
        super::print_openrouter_models(models);
    }

    pub fn print_local_models_for_test() {
        super::print_local_models();
    }

    pub fn current_model_label() -> String {
        crate::config::DEFAULT_CLI_MODEL.to_string()
    }

    pub fn print_current_footer() {
        super::print_current_footer(crate::config::DEFAULT_CLI_MODEL);
    }
}

#[cfg(test)]
#[path = "models_cmd_kiss_cov_tests.rs"]
mod models_cmd_kiss_cov_tests;

#[cfg(test)]
#[path = "models_cmd_local_tests.rs"]
mod models_cmd_local_tests;
