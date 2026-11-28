#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryMetric {
    Uss,
    Rss,
    UssAndRss,
}

impl MemoryMetric {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Uss => "USS",
            Self::Rss => "RSS",
            Self::UssAndRss => "USS+RSS",
        }
    }

    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Self::Uss, Self::Uss) => Self::Uss,
            (Self::Rss, Self::Rss) => Self::Rss,
            _ => Self::UssAndRss,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemorySample {
    pub bytes: u64,
    pub metric: MemoryMetric,
}

impl MemorySample {
    #[must_use]
    pub const fn new(bytes: u64, metric: MemoryMetric) -> Self {
        Self { bytes, metric }
    }

    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            bytes: self.bytes.saturating_add(other.bytes),
            metric: self.metric.combine(other.metric),
        }
    }
}
