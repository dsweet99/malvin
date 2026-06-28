//! Malvin-side mini agent client (`OpenRouter` + bash loop).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin_mini::OpenRouterClient;

use super::bash_adapter::ensure_bash_on_path;
use super::client_gate_retry::run_coder_prompt_with_gate_retries;
use super::client_prompt_log::{write_prompt_log, PromptLogWrite};
use super::loop_driver::{LoopDriverConfig, LoopDriverSession, LlmBackend};
use super::model_resolve::resolve_mini_model;
use super::retry_fork::MiniRetryStrategy;
use super::trace::MiniTraceSink;
use crate::acp::{AgentError, AgentIoOptions, AuthError, CoderPromptOptions};
use crate::prompts::default_file;

pub struct MiniLoopConfig {
    pub model: String,
    pub max_http_turns: u32,
    pub max_bash_execs: u32,
    pub max_http_retries: u32,
    pub max_gate_retries: u32,
    pub max_shrink_passes: u32,
    pub retry_strategy: MiniRetryStrategy,
    pub expects_investigation: bool,
}

pub struct MiniAgentClient {
    pub config: MiniLoopConfig,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub(crate) llm: LlmBackend,
    pub(crate) session: Option<LoopDriverSession>,
    pub(crate) last_response: Option<String>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    pub(crate) trace: MiniTraceSink,
    prompt_counter: u32,
}

impl MiniAgentClient {
    pub fn new(config: MiniLoopConfig, io: AgentIoOptions) -> Result<Self, String> {
        ensure_bash_on_path()?;
        let openrouter_config =
            malvin_mini::OpenRouterConfig::from_env(resolve_mini_model(&config.model))?;
        let client = OpenRouterClient::new(openrouter_config)
            .map_err(|e| format!("OpenRouter client init failed: {e}"))?;
        Ok(Self {
            config,
            io,
            prompts_log_run_dir: None,
            llm: LlmBackend::Http(client),
            session: None,
            last_response: None,
            timing: None,
            trace: MiniTraceSink::new(None, io),
            prompt_counter: 0,
        })
    }

    #[cfg(test)]
    pub const fn new_mock(config: MiniLoopConfig, io: AgentIoOptions, llm: LlmBackend) -> Self {
        Self {
            config,
            io,
            prompts_log_run_dir: None,
            llm,
            session: None,
            last_response: None,
            timing: None,
            trace: MiniTraceSink::new(None, io),
            prompt_counter: 0,
        }
    }

    /// # Errors
    ///
    /// Returns [`AuthError`] when `OPENROUTER_API_KEY` is missing.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            return Ok(());
        }
        Err(AuthError(
            "OPENROUTER_API_KEY is not set (required for --mini)".into(),
        ))
    }

    #[must_use]
    pub const fn has_open_coder_session(&self) -> bool {
        self.session.is_some()
    }

    #[must_use]
    pub const fn max_acp_retries(&self) -> u32 {
        self.config.max_gate_retries
    }

    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.session.is_some() {
            return Err(AgentError("mini coder session is already open".into()));
        }
        ensure_bash_on_path().map_err(AgentError)?;
        crate::malvin_sandbox::note_active_mini_session(cwd).map_err(AgentError)?;
        self.trace.run_dir = self.prompts_log_run_dir.clone();
        self.session = Some(LoopDriverSession {
            messages: vec![],
            cwd: cwd.to_path_buf(),
            constraints_prepended: false,
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: resolve_mini_model(&self.config.model),
        });
        self.prompt_counter = 0;
        Ok(())
    }

    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        self.session = None;
        self.last_response = None;
        crate::malvin_sandbox::clear_active_mini_session();
        Ok(())
    }

    #[must_use]
    pub fn last_coder_prompt_agent_response(&self) -> Option<String> {
        self.last_response.clone()
    }

    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        if self.session.is_none() {
            return Err(AgentError("begin_coder_session was not called".into()));
        }

        crate::prompts::enforce_no_unresolved_braces_in(prompt, opts.stdout_bracket_label)
            .map_err(|e| AgentError(e.0))?;

        let mini_constraints = default_file("mini_constraints.md").unwrap_or("");
        let effective_prompt = if self.session.as_ref().is_some_and(|s| !s.constraints_prepended) {
            format!("{mini_constraints}\n\n{prompt}")
        } else {
            prompt.to_string()
        };

        self.trace.plain_lines = opts.do_trace_split.is_some();

        write_prompt_log(PromptLogWrite {
            client: self,
            prompt: &effective_prompt,
            log_path,
            who,
            opts: &opts,
        })?;

        let driver_config = LoopDriverConfig {
            max_http_turns: self.config.max_http_turns,
            max_bash_execs: self.config.max_bash_execs,
            max_http_retries: self.config.max_http_retries,
            max_shrink_passes: self.config.max_shrink_passes,
            mini_constraints,
            expects_investigation: self.config.expects_investigation,
        };

        self.trace.log_outgoing_prompt(&effective_prompt);

        let session = self.session.as_mut().expect("session checked above");
        session.prompt_index = self.prompt_counter;
        self.prompt_counter += 1;

        run_coder_prompt_with_gate_retries(self, prompt, &driver_config, opts).await
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use crate::agent_backend::mini::{LlmBackend, MockScript, MockStep};

    #[test]
    fn mini_new_mock_skips_openrouter_init() {
        let client = MiniAgentClient::new_mock(
            MiniLoopConfig {
                model: "m".into(),
                max_http_turns: 4,
                max_bash_execs: 128,
                max_http_retries: 1,
                max_gate_retries: 1,
                max_shrink_passes: 0,
                retry_strategy: MiniRetryStrategy::CumulativeTranscript,
                expects_investigation: false,
            },
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            LlmBackend::Mock(std::sync::Mutex::new(MockScript {
                responses: vec![MockStep::Ok(malvin_mini::CompletionResponse {
                    content: "ok".into(),
                    usage: None,
                    reasoning: None,
                })],
                call_count: 0,
                on_response: None,
            })),
        );
        assert!(!client.has_open_coder_session());
    }
}
