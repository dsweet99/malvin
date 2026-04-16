#[derive(Debug, Clone)]
pub enum MultiturnPrompt {
    KpopBlock(String),
    Mbc2(String),
}

impl MultiturnPrompt {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::KpopBlock(s) | Self::Mbc2(s) => s.as_str(),
        }
    }
}
