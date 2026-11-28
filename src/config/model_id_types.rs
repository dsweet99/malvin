pub const CURSOR_PREFIX: &str = "cursor:";
pub const PI_PREFIX: &str = "pi:";
pub const CODEX_PREFIX: &str = "codex:";
pub const CLAUDE_PREFIX: &str = "claude:";

pub const MINI_PREFIX: &str = "mini:";
pub const RPI_PREFIX: &str = "rpi:";
pub const OPENROUTER_PREFIX: &str = "openrouter:";
pub const LOCAL_PREFIX: &str = "local:";
pub const PRIME_PREFIX: &str = "prime:";

pub const UNPREFIXED_MODEL_MESSAGE: &str = "model id must use a `cursor:`, `pi:`, `codex:`, or `claude:` prefix (for example `cursor:auto`, `pi:openai/gpt-4o`, `codex:gpt-5.6`, or `claude:sonnet`)";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelParam {
    pub id: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelBackend {
    Cursor,
    Pi,
    Codex,
    Claude,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModel {
    pub backend: ModelBackend,
    pub slug: String,
    pub params: Vec<ModelParam>,
}
