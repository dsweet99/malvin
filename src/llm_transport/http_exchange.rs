//! HTTP exchange metadata for local completion mapping.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpExchangeMeta {
    pub status: Option<u16>,
    pub body: Option<String>,
}
