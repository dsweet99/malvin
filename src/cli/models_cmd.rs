use malvin::model_id::{CODEX_PREFIX, CURSOR_PREFIX, PI_PREFIX, RPI_PREFIX};
use malvin::output::{MALVIN_WHO, print_stdout_line};
use clap::Args;

#[path = "models_cmd_cursor.rs"]
mod models_cmd_cursor;
#[path = "models_cmd_filter.rs"]
mod models_cmd_filter;
#[path = "models_cmd_parse.rs"]
mod models_cmd_parse;
#[path = "models_cmd_refresh.rs"]
pub(crate) mod models_cmd_refresh;
use models_cmd_cursor::print_cursor_models;
pub(crate) use models_cmd_filter::{line_matches_prefix, models_list_prefix, section_may_match};

#[derive(Args, Debug, Clone, Default)]
#[command(override_usage = "malvin admin models [OPTION]... [PREFIX]...")]
pub struct ModelsArgs {
    /// Force-refresh `pi:` and `rpi:` model catalogs (also runs automatically every 24h).
    #[arg(long)]
    pub refresh: bool,
    /// Optional prefix filter (for example `cursor:`, `pi:`, `rpi:`, or `codex:`)
    #[arg(
        value_name = "PREFIX",
        trailing_var_arg = true,
        allow_hyphen_values = true
    )]
    pub words: Vec<String>,
}

#[cfg(test)]
pub(crate) const fn models_args_marker(_args: &ModelsArgs) -> &'static str { "models" }

fn print_codex_models(filter: Option<&str>) {
    match malvin::codex_sdk::list_codex_display_models() {
        Ok(models) => {
            for (id, name) in models {
                let line = format!("codex:{id}\t{name}");
                if line_matches_prefix(&line, filter) {
                    print_stdout_line(MALVIN_WHO, &line);
                }
            }
        }
        Err(e) => print_stdout_line(MALVIN_WHO, &format!("(codex models unavailable: {e})")),
    }
}

fn print_current_footer(current_model: &str) {
    print_stdout_line(MALVIN_WHO, "");
    print_stdout_line(MALVIN_WHO, &format!("Current: {current_model}"));
}

pub fn run_models(args: ModelsArgs, current_model: &str) -> Result<(), String> {
    let filter = models_list_prefix(&args.words)?;
    let filter_ref = filter.as_deref();

    let now = models_cmd_refresh::unix_now_secs();
    let force_refresh = args.refresh || models_cmd_refresh::models_refresh_is_due(now);
    if force_refresh {
        models_cmd_refresh::perform_models_refresh();
    }

    if section_may_match(filter_ref, CURSOR_PREFIX)
        && let Err(e) = print_cursor_models(filter_ref)
    {
        print_stdout_line(MALVIN_WHO, &format!("(cursor models unavailable: {e})"));
    }
    if section_may_match(filter_ref, PI_PREFIX) {
        match malvin::npm_pi_sdk::list_npm_pi_display_models() {
            Ok(models) => print_npm_pi_models(&models, filter_ref),
            Err(e) => {
                print_stdout_line(MALVIN_WHO, &format!("(pi models unavailable: {e})"));
            }
        }
    }
    if section_may_match(filter_ref, RPI_PREFIX) {
        match malvin::pi_sdk::list_pi_models_sync(false) {
            Ok(models) => print_pi_models(&models, filter_ref),
            Err(e) => {
                print_stdout_line(MALVIN_WHO, &format!("(rpi models unavailable: {e})"));
            }
        }
    }
    if section_may_match(filter_ref, CODEX_PREFIX) {
        print_codex_models(filter_ref);
    }
    print_current_footer(current_model);
    Ok(())
}

fn print_npm_pi_models(models: &[(String, String)], filter: Option<&str>) {
    for (id, detail) in models {
        let line = format!("{PI_PREFIX}{id}\t{detail}");
        if line_matches_prefix(&line, filter) {
            print_stdout_line(MALVIN_WHO, &line);
        }
    }
}

fn print_pi_models(models: &[malvin::pi_sdk::PiModelListing], filter: Option<&str>) {
    let mut printed = false;
    for model in models {
        let provider = model.id.split('/').next().unwrap_or("");
        if !malvin::pi_sdk::is_provider_authenticated(provider) {
            continue;
        }
        let mut line = format!("{RPI_PREFIX}{}\t{}", model.id, model.name);
        if let Some(thinking) = model.thinking {
            line.push('\t');
            line.push_str(if thinking {
                "thinking=yes"
            } else {
                "thinking=no"
            });
        }
        if line_matches_prefix(&line, filter) {
            print_stdout_line(MALVIN_WHO, &line);
            printed = true;
        }
    }
    if printed {
        print_stdout_line(
            MALVIN_WHO,
            "Note: pi:/rpi: model lists refresh live provider catalogs at most once per day (use --refresh to force); rpi: rows are shown only for providers you can run (environment API key, stored Pi credential, Pi-detected local CLI auth such as Codex, keyless local provider, or keyless models.json entry). Older malvin ≤0.2.3 listed every provider; cargo install of 0.2.4+ needs rustc 1.95+.",
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
        models_cmd_parse::print_parsed_or_fallback_prefixed(text, "", None);
    }

    pub fn parse_model_line(line: &str) -> Option<(&str, String)> {
        models_cmd_parse::parse_model_line(line)
    }

    pub fn print_cursor_models_via_cli_for_test(filter: Option<&str>) -> Result<(), String> {
        super::models_cmd_cursor::print_cursor_models_via_cli(filter)
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
