
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptStratum {
    EmbeddedTemplate,
    PlaceholderContext,
    WorkflowHeader,
    UserRequest,
    GateLoopBlock,
    MiniConstraints,
}

#[must_use]
pub fn join_strata<I, S>(parts: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let trimmed: Vec<String> = parts
        .into_iter()
        .map(|p| p.as_ref().trim_end().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    trimmed.join("\n\n")
}

#[must_use]
pub fn join_labeled_strata<I, S>(parts: I) -> String
where
    I: IntoIterator<Item = (PromptStratum, S)>,
    S: AsRef<str>,
{
    join_strata(parts.into_iter().map(|(_, s)| s))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowRenderContext(pub HashMap<String, String>);

impl WorkflowRenderContext {
    #[must_use]
    pub const fn new(map: HashMap<String, String>) -> Self {
        Self(map)
    }

    #[must_use]
    pub const fn as_map(&self) -> &HashMap<String, String> {
        &self.0
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.0.insert(key.into(), value.into());
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0.get(key)
    }

    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

impl From<HashMap<String, String>> for WorkflowRenderContext {
    fn from(map: HashMap<String, String>) -> Self {
        Self(map)
    }
}

#[cfg(test)]
#[path = "prompt_stratification_tests.rs"]
mod prompt_stratification_tests;
