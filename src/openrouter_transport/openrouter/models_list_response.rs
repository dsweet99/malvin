use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct ModelsListRow {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelsListResponse {
    pub data: Vec<ModelsListRow>,
}

