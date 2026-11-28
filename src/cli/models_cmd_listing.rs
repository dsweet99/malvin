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

    pub(super) fn unavailable(error: String) -> Self {
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
