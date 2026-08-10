//! `malvin models` — list Cursor and Prime models with prefixes.

use crate::local_llm::local_model_listings;
use crate::model_id::{CURSOR_PREFIX, PRIME_PREFIX};
use crate::output::{MALVIN_WHO, print_stdout_line};
use clap::Args;

#[path = "models_cmd_parse.rs"]
mod models_cmd_parse;
#[path = "models_cmd_cursor.rs"]
mod models_cmd_cursor;
#[path = "models_cmd_filter.rs"]
mod models_cmd_filter;
use models_cmd_cursor::print_cursor_models;
pub(crate) use models_cmd_filter::{
    line_matches_prefix, models_list_prefix, section_may_match,
};

#[derive(Args, Debug, Clone, Default)]
pub struct ModelsArgs {
    /// Optional prefix filter (e.g. `prime:`, `prime:open`, `prime:local/`). See `models_list_prefix`.
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

/// Print Cursor and Prime models with prefixes and a `Current:` footer.
///
/// Optional `words` form a prefix filter on printed model ids (see [`models_list_prefix`]).
pub fn run_models(args: ModelsArgs, current_model: &str) -> Result<(), String> {
    let filter = models_list_prefix(&args.words)?;
    let filter_ref = filter.as_deref();

    if section_may_match(filter_ref, CURSOR_PREFIX) {
        if let Err(e) = print_cursor_models(filter_ref) {
            print_stdout_line(MALVIN_WHO, &format!("(cursor models unavailable: {e})"));
        }
    }
    if section_may_match(filter_ref, PRIME_PREFIX) {
        match crate::prime_sdk::list_prime_models_sync() {
            Ok(models) => print_prime_models(&models, filter_ref),
            Err(e) => {
                print_stdout_line(MALVIN_WHO, &format!("(prime models unavailable: {e})"));
            }
        }
        print_prime_local_models(filter_ref);
    }
    print_current_footer(current_model);
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

const PRIME_LOCAL_HEAD: &str = "prime:local/";

fn print_prime_local_models(filter: Option<&str>) {
    for model in local_model_listings() {
        let line = format!("{PRIME_LOCAL_HEAD}{}\t{}", model.id, model.name);
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

    pub fn sdk_model_rows_from_stdout(raw: &str) -> Vec<String> {
        super::models_cmd_cursor::sdk_model_rows_from_stdout(raw)
    }

    pub fn models_display_lines_filtered(
        text: &str,
        prefix: &str,
        filter: Option<&str>,
    ) -> Option<Vec<String>> {
        models_cmd_parse::models_display_lines_filtered(text, prefix, filter)
    }

    pub fn print_local_models_for_test() {
        super::print_prime_local_models(None);
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
