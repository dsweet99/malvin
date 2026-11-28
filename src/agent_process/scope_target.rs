use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeTarget {
    Unit(String),
    Nested(PathBuf),
}

#[derive(Debug)]
pub struct SandboxSpawnTicket {
    pub(crate) scope_unit: Option<ScopeTarget>,
}

impl SandboxSpawnTicket {
    pub(crate) fn set_scope_unit(&mut self, unit: Option<ScopeTarget>) {
        self.scope_unit = unit;
    }

    #[cfg(test)]
    pub(crate) const fn for_test() -> Self {
        Self { scope_unit: None }
    }
}
