//! `[review]` section parsing for `config.toml`.

use super::ReviewConfig;

pub(crate) fn parse_review_config(text: &str) -> Result<ReviewConfig, String> {
    let value: toml::Value = text
        .parse()
        .map_err(|e| format!("invalid TOML: {e}"))?;
    let Some(review) = value.get("review") else {
        return Ok(ReviewConfig::default());
    };
    Ok(ReviewConfig {
        max_hypotheses: super::read_usize(review.get("max_hypotheses")),
    })
}
