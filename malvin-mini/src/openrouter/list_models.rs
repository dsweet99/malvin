use serde::Deserialize;

use crate::error::OpenRouterError;

use super::client::{build_catalog_request_headers, OpenRouterClient};
use super::complete::map_http_status;
use super::models_list_response::ModelsListResponse;

/// One row from OpenRouter `GET /models`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ModelListing {
    pub id: String,
    pub name: String,
}

pub(crate) fn list_models_url(base_url: &str) -> String {
    format!(
        "{}/models?output_modalities=text&sort=most-popular",
        base_url.trim_end_matches('/')
    )
}

impl OpenRouterClient {
    /// # Errors
    ///
    /// Returns [`OpenRouterError`] on HTTP, status, or JSON failures.
    pub async fn list_models(&self) -> Result<Vec<ModelListing>, OpenRouterError> {
        let url = list_models_url(&self.config().base_url);
        let headers = build_catalog_request_headers(self.config())?;
        let resp = self.http().get(url).headers(headers).send().await?;
        let status = resp.status().as_u16();
        let text = resp.text().await?;
        map_http_status(status, &text)?;
        let parsed: ModelsListResponse = serde_json::from_str(&text)?;
        Ok(parsed
            .data
            .into_iter()
            .map(|row| ModelListing {
                id: row.id,
                name: row.name,
            })
            .collect())
    }
}
