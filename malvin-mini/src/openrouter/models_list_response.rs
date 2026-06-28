use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct ModelsListRow {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ModelsListResponse {
    pub data: Vec<ModelsListRow>,
}

#[cfg(test)]
#[path = "models_list_response_tests.rs"]
mod models_list_response_tests;
