//! `malvin models` — list Cursor, Prime, Mini `OpenRouter`, and Mini local models with prefixes.

use crate::local_llm::local_model_listings;
use crate::model_id::{CURSOR_PREFIX, MINI_PREFIX, PRIME_PREFIX};
use crate::output::{MALVIN_WHO, print_stdout_line};
use clap::Args;

#[path = "models_cmd_parse.rs"]
mod models_cmd_parse;
#[path = "models_cmd_cursor.rs"]
mod models_cmd_cursor;
use models_cmd_cursor::print_cursor_models;

const MINI_OPENROUTER_HEAD: &str = "mini:openrouter/";
const MINI_LOCAL_HEAD: &str = "mini:local/";

#[derive(Args, Debug, Clone, Default)]
pub struct ModelsArgs {
    /// Optional prefix filter (e.g. `prime:`, `prime:open`, `mini:local/`). Words are concatenated.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub words: Vec<String>,
}

#[cfg(test)]
pub(crate) const fn models_args_marker(_args: &ModelsArgs) -> &'static str {
    "models"
}

fn print_current_footer(current_model: &str) {
    print_stdout_line(MALVIN_WHO, "");
    print_stdout_line(MALVIN_WHO, &format!("Current: {current_model}"));
}

/// Print Cursor, Prime, Mini `OpenRouter`, and Mini local models with prefixes and a `Current:` footer.
///
/// Optional `words` form a prefix filter on printed model ids (concatenated, no separators).
pub fn run_models(args: ModelsArgs, current_model: &str) -> Result<(), String> {
    let filter = models_list_prefix(&args.words)?;
    let filter_ref = filter.as_deref();

    if section_may_match(filter_ref, CURSOR_PREFIX) {
        print_cursor_models(filter_ref)?;
    }
    if section_may_match(filter_ref, PRIME_PREFIX) {
        match crate::prime_sdk::list_prime_models_sync() {
            Ok(models) => print_prime_models(&models, filter_ref),
            Err(e) => {
                print_stdout_line(MALVIN_WHO, &format!("(prime models unavailable: {e})"));
            }
        }
    }
    if section_may_match(filter_ref, MINI_OPENROUTER_HEAD) {
        match list_openrouter_models_sync() {
            Ok(models) => print_openrouter_models(&models, filter_ref),
            Err(e) => {
                print_stdout_line(
                    MALVIN_WHO,
                    &format!("(mini:openrouter models unavailable: {e})"),
                );
            }
        }
    }
    if section_may_match(filter_ref, MINI_LOCAL_HEAD) {
        print_local_models(filter_ref);
    }
    print_current_footer(current_model);
    Ok(())
}

/// Resolve optional listing prefix from trailing words.
///
/// Rejects legacy `download …` action words. Multiple words are concatenated with no separator
/// so `malvin models prime: open` matches the same as `malvin models prime:open`.
pub(crate) fn models_list_prefix(words: &[String]) -> Result<Option<String>, String> {
    if words.is_empty() {
        return Ok(None);
    }
    if words[0].eq_ignore_ascii_case("download") {
        return Err(format!(
            "`malvin models` no longer downloads; `{MINI_PREFIX}local/…` models fetch automatically on first use (omit `--no-download`)"
        ));
    }
    Ok(Some(words.join("")))
}

/// Whether a catalog section whose ids start with `section_head` can produce rows for `filter`.
pub(crate) fn section_may_match(filter: Option<&str>, section_head: &str) -> bool {
    match filter {
        None => true,
        Some("") => true,
        Some(f) => f.starts_with(section_head) || section_head.starts_with(f),
    }
}

/// Whether a printed model row matches an optional id prefix filter.
pub(crate) fn line_matches_prefix(line: &str, filter: Option<&str>) -> bool {
    let Some(f) = filter else {
        return true;
    };
    if f.is_empty() {
        return true;
    }
    let id = line.split('\t').next().unwrap_or(line).trim();
    id.starts_with(f)
}

fn list_openrouter_models_sync() -> Result<Vec<crate::openrouter_transport::ModelListing>, String> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("failed to create Tokio runtime: {e}"))?;
    rt.block_on(async {
        use crate::openrouter_transport::{OpenRouterClient, OpenRouterConfig};
        let config = OpenRouterConfig::from_env_for_listing()?;
        let client = OpenRouterClient::new(config).map_err(|e| e.to_string())?;
        client.list_models().await.map_err(|e| e.to_string())
    })
}

/// Fetch `OpenRouter` models (async helper for tests).
#[cfg(any(test, doctest))]
pub async fn run_mini_models() -> Result<(), String> {
    use crate::openrouter_transport::{OpenRouterClient, OpenRouterConfig};

    let config = OpenRouterConfig::from_env_for_listing()?;
    let client = OpenRouterClient::new(config).map_err(|e| e.to_string())?;
    let models = client.list_models().await.map_err(|e| e.to_string())?;
    print_openrouter_models(&models, None);
    print_current_footer(crate::config::DEFAULT_CLI_MODEL);
    Ok(())
}

fn print_prime_models(models: &[crate::prime_sdk::PrimeModelListing], filter: Option<&str>) {
    for model in models {
        let line = format!("prime:{}\t{}", model.id, model.name);
        if line_matches_prefix(&line, filter) {
            print_stdout_line(MALVIN_WHO, &line);
        }
    }
}

fn print_openrouter_models(
    models: &[crate::openrouter_transport::ModelListing],
    filter: Option<&str>,
) {
    for model in models {
        let line = format!("{MINI_PREFIX}openrouter/{}\t{}", model.id, model.name);
        if line_matches_prefix(&line, filter) {
            print_stdout_line(MALVIN_WHO, &line);
        }
    }
}

fn print_local_models(filter: Option<&str>) {
    for model in local_model_listings() {
        let line = format!("{MINI_PREFIX}local/{}\t{}", model.id, model.name);
        if line_matches_prefix(&line, filter) {
            print_stdout_line(MALVIN_WHO, &line);
        }
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
        models_cmd_parse::print_parsed_or_fallback_prefixed(text, "", None);
    }

    pub fn parse_model_line(line: &str) -> Option<(&str, String)> {
        models_cmd_parse::parse_model_line(line)
    }

    pub fn resolve_models_cli() -> Result<std::path::PathBuf, String> {
        super::models_cmd_cursor::resolve_models_cli()
    }

    pub fn print_mini_models(models: &[crate::openrouter_transport::ModelListing]) {
        super::print_openrouter_models(models, None);
    }

    pub fn print_local_models_for_test() {
        super::print_local_models(None);
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
