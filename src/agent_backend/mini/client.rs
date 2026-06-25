//! Malvin-side mini agent client (`OpenRouter` + bash loop).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use malvin_mini::OpenRouterClient;

use super::bash_adapter::ensure_bash_on_path;
use super::client_prompt_log::{write_prompt_log, PromptLogWrite};
use super::loop_driver::{
    run_inner_loop, LoopDriverConfig, LoopDriverRun, LoopDriverSession, LlmBackend,
};
use super::model_resolve::resolve_mini_model;
use super::trace::MiniTraceSink;
use crate::acp::{
    backoff_after_agent_failure, retries_noun, AgentError, AgentIoOptions, AuthError,
    CoderPromptOptions,
};
use crate::prompts::default_file;

pub struct MiniLoopConfig {
    pub model: String,
    pub max_bash_turns: u32,
    pub max_http_retries: u32,
}

pub struct MiniAgentClient {
    pub config: MiniLoopConfig,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    llm: LlmBackend,
    session: Option<LoopDriverSession>,
    last_response: Option<String>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    trace: MiniTraceSink,
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
            trace: MiniTraceSink {
                run_dir: None,
                io,
            },
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
            trace: MiniTraceSink {
                run_dir: None,
                io,
            },
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
        self.config.max_http_retries
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
        });
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

        write_prompt_log(PromptLogWrite {
            client: self,
            prompt,
            log_path,
            who,
            opts: &opts,
        })?;

        let driver_config = LoopDriverConfig {
            max_bash_turns: self.config.max_bash_turns,
            max_http_retries: self.config.max_http_retries,
            mini_constraints: default_file("mini_constraints.md").unwrap_or(""),
        };

        self.run_coder_prompt_with_retries(prompt, &driver_config, opts).await
    }

    async fn run_coder_prompt_with_retries(
        &mut self,
        prompt: &str,
        driver_config: &LoopDriverConfig,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        let max_attempts = if opts.single_attempt {
            1
        } else {
            self.config.max_http_retries.max(1)
        };
        let mut last_error = String::new();
        let mut attempts_used = 0_u32;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            let session = self.session.as_mut().expect("session checked above");
            let message_checkpoint = session.messages.len();
            match run_inner_loop(LoopDriverRun {
                llm: &self.llm,
                session,
                user_prompt: prompt,
                config: driver_config,
                trace: &self.trace,
                timing: self.timing.as_ref(),
                llm_phase: opts.llm_phase,
                single_attempt: opts.single_attempt,
            })
            .await
            {
                Ok(outcome) => {
                    self.last_response = Some(outcome.final_assistant_text);
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.0;
                    session.messages.truncate(message_checkpoint);
                    if opts.single_attempt {
                        return Err(AgentError(last_error));
                    }
                    if backoff_after_agent_failure(
                        self.timing.as_ref(),
                        &last_error,
                        attempt,
                        max_attempts,
                    )
                    .await?
                    {
                        break;
                    }
                }
            }
        }
        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "mini agent (gate_iteration) failed after {retries} {noun}. Last error:\n{last_error}"
        )))
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
                max_bash_turns: 4,
                max_http_retries: 1,
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
                })],
                call_count: 0,
                on_response: None,
            })),
        );
        assert!(!client.has_open_coder_session());
    }
}
