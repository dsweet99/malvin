#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Router,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaggedRequest {
    pub text: String,
    pub kind: RequestKind,
}

impl TaggedRequest {
    #[must_use]
    pub const fn is_router(&self) -> bool {
        matches!(self.kind, RequestKind::Router)
    }
}

pub(crate) fn tag_requests(requests: &[String]) -> Vec<TaggedRequest> {
    requests
        .iter()
        .map(|text| TaggedRequest {
            text: text.clone(),
            kind: RequestKind::Router,
        })
        .collect()
}
