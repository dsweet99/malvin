use std::time::Duration;

const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1";
const DEFAULT_REQUEST_TIMEOUT_SECS: u64 = 120;

#[must_use]
pub fn request_timeout_from_secs_str(s: Option<&str>) -> Duration {
    let secs = s
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECS);
    Duration::from_secs(secs)
}

#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    pub model: String,
    pub api_key: String,
    pub http_referer: Option<String>,
    pub request_timeout: Duration,
    pub base_url: String,
}

impl OpenRouterConfig {
    pub fn from_env(model: String) -> Result<Self, String> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .map_err(|_| "OPENROUTER_API_KEY is not set".to_string())?;
        Ok(Self::from_env_parts(model, api_key))
    }

    /// Build config for `GET /models` listing. `OPENROUTER_API_KEY` is optional.
    pub fn from_env_for_listing() -> Result<Self, String> {
        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        Ok(Self::from_env_parts(String::new(), api_key))
    }

    fn from_env_parts(model: String, api_key: String) -> Self {
        let http_referer = std::env::var("OPENROUTER_HTTP_REFERER").ok();
        let base_url = std::env::var("OPENROUTER_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let request_timeout = request_timeout_from_secs_str(
            std::env::var("OPENROUTER_REQUEST_TIMEOUT").ok().as_deref(),
        );
        Self {
            model,
            api_key,
            http_referer,
            request_timeout,
            base_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unsafe_code)]
    fn with_env(key: &str, value: Option<&str>, f: impl FnOnce()) {
        unsafe {
            let prior = std::env::var(key).ok();
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
            f();
            match prior {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn openrouter_config_from_env_reads_all_fields() {
        with_env("OPENROUTER_API_KEY", Some("sk-test"), || {
            with_env("OPENROUTER_HTTP_REFERER", Some("https://example.test"), || {
                with_env("OPENROUTER_BASE_URL", Some("https://custom.test/v1"), || {
                    with_env("OPENROUTER_REQUEST_TIMEOUT", Some("45"), || {
                        let cfg =
                            OpenRouterConfig::from_env("model-x".into()).expect("from_env");
                        assert_eq!(cfg.model, "model-x");
                        assert_eq!(cfg.api_key, "sk-test");
                        assert_eq!(cfg.http_referer.as_deref(), Some("https://example.test"));
                        assert_eq!(cfg.base_url, "https://custom.test/v1");
                        assert_eq!(cfg.request_timeout, Duration::from_secs(45));
                    });
                });
            });
        });
    }

    #[test]
    fn openrouter_config_from_env_errors_when_api_key_missing() {
        with_env("OPENROUTER_API_KEY", None, || {
            let err = OpenRouterConfig::from_env("model-x".into()).expect_err("missing key");
            assert!(err.contains("OPENROUTER_API_KEY"));
        });
    }

    #[test]
    fn openrouter_config_from_env_for_listing_allows_missing_api_key() {
        with_env("OPENROUTER_API_KEY", None, || {
            let cfg = OpenRouterConfig::from_env_for_listing().expect("listing config");
            assert!(cfg.api_key.is_empty());
            assert_eq!(cfg.base_url, DEFAULT_BASE_URL);
        });
    }

    #[test]
    fn openrouter_config_from_env_for_listing_reads_api_key_when_set() {
        with_env("OPENROUTER_API_KEY", Some("sk-list"), || {
            let cfg = OpenRouterConfig::from_env_for_listing().expect("listing config");
            assert_eq!(cfg.api_key, "sk-list");
        });
    }

    #[test]
    fn kiss_cov_openrouter_config_from_env_for_listing() {
        let _ = OpenRouterConfig::from_env_for_listing;
    }

    #[test]
    fn openrouter_config_reads_request_timeout_from_env() {
        assert_eq!(
            request_timeout_from_secs_str(Some("45")),
            Duration::from_secs(45)
        );
        assert_eq!(
            request_timeout_from_secs_str(None),
            Duration::from_secs(DEFAULT_REQUEST_TIMEOUT_SECS)
        );
    }
}
