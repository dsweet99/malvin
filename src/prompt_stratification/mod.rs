use std::collections::HashMap;

/// One host send that joins the workflow's initial prompt pieces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AggregatedInitialPrompt {
    pub body: String,
    pub stdout_label: String,
    pub log_who: &'static str,
}

/// Collect nonempty labeled pieces, then join them for a single host prompt.
#[derive(Debug, Default)]
pub struct AggregatedInitialPromptBuilder {
    strata: Vec<(PromptStratum, String)>,
    labels: Vec<&'static str>,
}

impl AggregatedInitialPromptBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_nonempty(&mut self, label: &'static str, body: String) {
        if body.is_empty() {
            return;
        }
        self.strata.push((PromptStratum::WorkflowHeader, body));
        self.labels.push(label);
    }

    #[must_use]
    pub fn finish(self, log_who: &'static str) -> AggregatedInitialPrompt {
        AggregatedInitialPrompt {
            body: join_labeled_strata(self.strata),
            stdout_label: self.labels.join("+"),
            log_who,
        }
    }
}

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
