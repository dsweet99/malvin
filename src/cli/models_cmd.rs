use clap::Args;
use malvin::output::{MALVIN_WHO, print_stdout_line};

pub(crate) use super::models_cmd_catalog as models_cmd_catalog;
pub(crate) use super::models_cmd_cursor as models_cmd_cursor;
pub(crate) use super::models_cmd_filter as models_cmd_filter;
pub(crate) use super::models_cmd_parse as models_cmd_parse;
pub(crate) use super::models_cmd_refresh as models_cmd_refresh;
pub(crate) use super::models_cmd_resolve as models_cmd_resolve;
use models_cmd_catalog::print_models_sections;
pub(crate) use models_cmd_filter::{line_matches_prefix, models_list_prefix, section_may_match};

#[derive(Args, Debug, Clone, Default)]
#[command(override_usage = "malvin admin models [OPTION]... [PREFIX]...")]
pub struct ModelsArgs {
    /// Force-refresh the `pi:` model catalog (also runs automatically every 24h).
    #[arg(long)]
    pub refresh: bool,
    /// Print MODEL's canonical id and backend as one JSON line (`[aliases.models]` names expanded), then exit.
    #[arg(long, value_name = "MODEL", conflicts_with_all = ["refresh", "words"])]
    pub resolve: Option<String>,
    /// Optional prefix filter (for example `cursor:`, `pi:`, or `codex:`)
    #[arg(
        value_name = "PREFIX",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
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

pub fn run_models(args: ModelsArgs, current_model: &str) -> Result<(), String> {
    if let Some(raw) = args.resolve.as_deref() {
        return models_cmd_resolve::write_resolved_model(raw, &mut std::io::stdout().lock());
    }
    let filter = models_list_prefix(&args.words)?;
    let filter_ref = filter.as_deref();
    maybe_refresh_models_catalog(args.refresh);
    print_models_sections(filter_ref);
    print_current_footer(current_model);
    Ok(())
}

fn maybe_refresh_models_catalog(force: bool) {
    let now = malvin::clock::unix_now_secs();
    if force || models_cmd_refresh::models_refresh_is_due(now) {
        models_cmd_refresh::perform_models_refresh();
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
        let listing = super::models_cmd_cursor::listing_from_cli_text(text, "");
        super::models_cmd_catalog::print_listing("", &listing, None);
    }

    pub fn parse_model_line(line: &str) -> Option<(&str, String)> {
        models_cmd_parse::parse_model_line(line)
    }

    pub fn print_cursor_models_via_cli_for_test(filter: Option<&str>) -> Result<(), String> {
        let listing = super::models_cmd_cursor::cursor_listing_via_cli()?;
        super::models_cmd_catalog::print_listing("cursor:", &listing, filter);
        Ok(())
    }

    pub fn resolve_models_cli() -> Result<std::path::PathBuf, String> {
        super::models_cmd_cursor::resolve_models_cli()
    }

    pub fn sdk_model_rows_from_stdout(raw: &str) -> Vec<String> {
        super::models_cmd_cursor::sdk_model_rows_from_stdout(raw)
    }

    pub fn sdk_catalog_has_model_rows(raw: &str) -> bool {
        super::models_cmd_cursor::sdk_catalog_has_model_rows(raw)
    }

    pub fn cursor_list_models_timeout() -> std::time::Duration {
        super::models_cmd_cursor::cursor_list_models_timeout()
    }

    pub fn models_display_lines_filtered(
        text: &str,
        prefix: &str,
        filter: Option<&str>,
    ) -> Option<Vec<String>> {
        models_cmd_parse::models_display_lines_filtered(text, prefix, filter)
    }

    pub fn current_model_label() -> String {
        malvin::config::DEFAULT_CLI_MODEL.to_string()
    }

    pub fn print_current_footer() {
        super::print_current_footer(malvin::config::DEFAULT_CLI_MODEL);
    }

    pub fn models_refresh_is_due(now_secs: u64) -> bool {
        super::models_cmd_refresh::models_refresh_is_due(now_secs)
    }

    pub fn save_last_refresh_secs(now_secs: u64) -> Result<(), String> {
        super::models_cmd_refresh::save_last_refresh_secs(now_secs)
    }

    pub fn load_last_refresh_secs() -> Option<u64> {
        super::models_cmd_refresh::load_last_refresh_secs()
    }
}

#[cfg(test)]
#[path = "models_cmd_kiss_cov_tests.rs"]
mod models_cmd_kiss_cov_tests;
