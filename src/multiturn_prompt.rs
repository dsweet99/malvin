#[derive(Debug, Clone)]
pub enum MultiturnPrompt {
    KpopBlock(String),
}

impl MultiturnPrompt {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::KpopBlock(s) => s.as_str(),
        }
    }
}
