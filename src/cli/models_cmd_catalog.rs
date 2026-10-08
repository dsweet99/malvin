use malvin::config::model_id::ModelBackend;
use malvin::output::{MALVIN_WHO, print_stdout_line, print_stdout_text};

use super::models_cmd::models_cmd_cursor::cursor_model_listing;
use super::models_cmd::{line_matches_prefix, section_may_match};

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ModelListing {
    pub(super) rows: Vec<String>,
    pub(super) unparsed: Option<String>,
    pub(super) unavailable: Option<String>,
}

impl ModelListing {
    pub(super) fn rows(rows: Vec<String>) -> Self {
        Self {
            rows,
            ..Self::default()
        }
    }

    fn unavailable(error: String) -> Self {
        Self {
            unavailable: Some(error),
            ..Self::default()
        }
    }
}

pub(super) trait ModelCatalog {
    fn prefix(&self) -> &'static str;
    fn list_display_models(&self) -> ModelListing;
}

struct CursorCatalog;
struct PiCatalog;
struct CodexCatalog;

impl ModelCatalog for CursorCatalog {
    fn prefix(&self) -> &'static str {
        ModelBackend::Cursor.spec().prefix()
    }

    fn list_display_models(&self) -> ModelListing {
        cursor_model_listing().unwrap_or_else(ModelListing::unavailable)
    }
}

impl ModelCatalog for PiCatalog {
    fn prefix(&self) -> &'static str {
        ModelBackend::Pi.spec().prefix()
    }

    fn list_display_models(&self) -> ModelListing {
        let (models, unavailable) = match malvin::backends::pi_backend::list_npm_pi_display_models()
        {
            Ok(models) => (models, None),
            Err(e) => (Vec::new(), Some(e)),
        };
        ModelListing {
            rows: prefixed_rows(
                self.prefix(),
                malvin::local_llm::filter_local_listings(models),
            ),
            unparsed: None,
            unavailable,
        }
    }
}

impl ModelCatalog for CodexCatalog {
    fn prefix(&self) -> &'static str {
        ModelBackend::Codex.spec().prefix()
    }

    fn list_display_models(&self) -> ModelListing {
        match malvin::backends::codex_sdk::list_codex_display_models() {
            Ok(models) => ModelListing::rows(prefixed_rows(self.prefix(), models)),
            Err(e) => ModelListing::unavailable(e),
        }
    }
}

const fn catalog_for(backend: ModelBackend) -> &'static dyn ModelCatalog {
    match backend {
        ModelBackend::Cursor => &CursorCatalog,
        ModelBackend::Pi => &PiCatalog,
        ModelBackend::Codex => &CodexCatalog,
    }
}

fn prefixed_rows(prefix: &str, models: Vec<(String, String)>) -> Vec<String> {
    models
        .into_iter()
        .map(|(id, detail)| format!("{prefix}{id}\t{detail}"))
        .collect()
}

pub(super) fn print_models_sections(filter: Option<&str>) {
    let mut shown = 0;
    for backend in ModelBackend::ALL {
        let catalog = catalog_for(backend);
        if section_may_match(filter, catalog.prefix()) {
            shown += print_listing(catalog.prefix(), &catalog.list_display_models(), filter);
        }
    }
    if let Some(f) = filter.filter(|f| !f.is_empty() && shown == 0) {
        print_stdout_line(MALVIN_WHO, &no_match_hint(f));
    }
}

fn no_match_hint(filter: &str) -> String {
    format!(
        "(no model id starts with `{filter}`; PREFIX matches the start of an id, such as `cursor:`, `pi:`, or `codex:`)"
    )
}

pub(super) fn print_listing(prefix: &str, listing: &ModelListing, filter: Option<&str>) -> usize {
    if let Some(e) = &listing.unavailable {
        let label = prefix.trim_end_matches(':');
        print_stdout_line(MALVIN_WHO, &format!("({label} models unavailable: {e})"));
    }
    let mut shown = 0;
    for row in listing
        .rows
        .iter()
        .filter(|row| line_matches_prefix(row, filter))
    {
        print_stdout_line(MALVIN_WHO, row);
        shown += 1;
    }
    if let Some(text) = &listing.unparsed
        && filter.is_none()
    {
        print_stdout_text(MALVIN_WHO, text);
        shown += 1;
    }
    shown
}

#[cfg(test)]
mod tests {
    use super::*;
    use malvin::output::{enable_stdout_capture, take_captured_stdout};

    #[test]
    fn every_backend_has_a_catalog_with_its_prefix() {
        for backend in ModelBackend::ALL {
            assert_eq!(catalog_for(backend).prefix(), backend.spec().prefix());
        }
    }

    #[test]
    fn print_listing_filters_rows_and_reports_unavailable() {
        let listing = ModelListing {
            rows: vec!["pi:a/b\tx".into(), "pi:c/d\ty".into()],
            unparsed: Some("raw text\n".into()),
            unavailable: Some("boom".into()),
        };
        enable_stdout_capture();
        assert_eq!(print_listing("pi:", &listing, Some("pi:a")), 1);
        assert_eq!(print_listing("pi:", &listing, Some("pi:z")), 0);
        assert!(no_match_hint("terra").contains("no model id starts with `terra`"));
        let filtered = take_captured_stdout();
        assert!(
            filtered.contains("(pi models unavailable: boom)"),
            "{filtered}"
        );
        assert!(filtered.contains("pi:a/b\tx"), "{filtered}");
        assert!(!filtered.contains("pi:c/d"), "{filtered}");
        assert!(!filtered.contains("raw text"), "{filtered}");
        enable_stdout_capture();
        print_listing("pi:", &listing, None);
        let all = take_captured_stdout();
        assert!(
            all.contains("pi:c/d\ty") && all.contains("raw text"),
            "{all}"
        );
    }
}
