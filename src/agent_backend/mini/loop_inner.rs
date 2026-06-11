//! Inner bash-fence loop for one `run_coder_prompt`.

use std::time::Instant;

use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

use crate::agent_backend::mini::bash_adapter::{format_observation, run_bash_command, BashExecResult};
use crate::agent_backend::mini::fence_parser::{parse_bash_fences, BashFence};
use super::loop_http::{complete_with_http_retries, HttpRetryRequest};
use super::loop_mock::LlmBackend;
use super::loop_types::{
    LoopDriverConfig, LoopDriverOutcome, LoopDriverRun, LoopDriverSession,
};
use crate::acp::AgentError;
use crate::run_timing::{record_llm, TimingPhase};

const NO_FENCE_NUDGE: &str = "your last response had no ```bash``` block";

struct CompleteTurnRequest<'a> {
    llm: &'a LlmBackend,
    session: &'a LoopDriverSession,
    config: &'a LoopDriverConfig,
    trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    llm_phase: Option<TimingPhase>,
    single_attempt: bool,
}

pub(crate) enum TurnAction {
    Done(String),
    Continue,
    RunBash(Vec<BashFence>),
}

/// Run the inner loop for one user prompt (already includes mini constraints when caller prepends).
///
/// # Errors
///
/// Returns [`AgentError`] when HTTP or bash budget is exhausted.
pub async fn run_inner_loop(run: LoopDriverRun<'_>) -> Result<LoopDriverOutcome, AgentError> {
    let LoopDriverRun {
        llm,
        session,
        user_prompt,
        config,
        trace,
        timing,
        llm_phase,
        single_attempt,
    } = run;
    push_user_prompt(session, config.mini_constraints, user_prompt);

    let mut transcript = String::new();
    let mut no_fence_nudge_used = false;
    let mut final_text = String::new();

    for turn in 0..config.max_bash_turns {
        let response = complete_turn(CompleteTurnRequest {
            llm,
            session,
            config,
            trace,
            timing,
            llm_phase,
            single_attempt,
        })
        .await?;
        let assistant_text = response.content.clone();
        transcript.push_str(&assistant_text);
        transcript.push('\n');
        session.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: assistant_text.clone(),
        });

        match classify_turn(&assistant_text, no_fence_nudge_used) {
            TurnAction::Done(text) => {
                final_text = text;
                trace.mini_assistant(&final_text);
                break;
            }
            TurnAction::Continue => {
                no_fence_nudge_used = true;
                session.messages.push(ChatMessage {
                    role: ChatRole::User,
                    content: NO_FENCE_NUDGE.into(),
                });
            }
            TurnAction::RunBash(fences) => {
                append_bash_observation(session, &fences, trace, &mut transcript)?;
                if turn + 1 >= config.max_bash_turns {
                    return Err(exhausted_error(config.max_bash_turns, &transcript));
                }
            }
        }
    }

    if final_text.is_empty() {
        return Err(exhausted_error(config.max_bash_turns, &transcript));
    }

    Ok(LoopDriverOutcome {
        final_assistant_text: final_text,
    })
}

pub(crate) fn push_user_prompt(session: &mut LoopDriverSession, mini_constraints: &str, user_prompt: &str) {
    let full_prompt = format!("{mini_constraints}\n\n{user_prompt}");
    session.messages.push(ChatMessage {
        role: ChatRole::User,
        content: full_prompt,
    });
}

async fn complete_turn(req: CompleteTurnRequest<'_>) -> Result<CompletionResponse, AgentError> {
    let CompleteTurnRequest {
        llm,
        session,
        config,
        trace,
        timing,
        llm_phase,
        single_attempt,
    } = req;
    crate::agent_phase::note_mini_llm_request();
    let t0 = Instant::now();
    let response = complete_with_http_retries(HttpRetryRequest {
        llm,
        messages: &session.messages,
        max_retries: config.max_http_retries,
        single_attempt,
        timing,
    })
    .await?;
    record_llm(timing, llm_phase.unwrap_or(TimingPhase::Implement), t0.elapsed());
    if let Some(ref usage) = response.usage {
        crate::run_timing::record_mini_http_cost(timing, usage);
    }
    trace.mini_llm_request(response.usage.as_ref());
    Ok(response)
}

pub(crate) fn classify_turn(assistant_text: &str, no_fence_nudge_used: bool) -> TurnAction {
    if assistant_text.lines().any(|l| l.trim() == "MINI_DONE") {
        return TurnAction::Done(assistant_text.to_string());
    }
    let fences = parse_bash_fences(assistant_text);
    if fences.is_empty() {
        if no_fence_nudge_used {
            TurnAction::Done(assistant_text.to_string())
        } else {
            TurnAction::Continue
        }
    } else {
        TurnAction::RunBash(fences)
    }
}

fn append_bash_observation(
    session: &mut LoopDriverSession,
    fences: &[BashFence],
    trace: &crate::agent_backend::mini::trace::MiniTraceSink,
    transcript: &mut String,
) -> Result<(), AgentError> {
    let mut results: Vec<BashExecResult> = Vec::new();
    for fence in fences {
        crate::agent_phase::note_mini_bash_exec();
        let t0 = Instant::now();
        let result = run_bash_command(session.cwd.as_path(), &fence.command).map_err(AgentError)?;
        let elapsed = t0.elapsed();
        trace.mini_bash_exec(&fence.command, result.exit_code, elapsed);
        crate::agent_phase::note_mini_bash_exec_done(result.exit_code, &fence.command);
        results.push(result);
    }
    let observation = format_observation(&results);
    session.messages.push(ChatMessage {
        role: ChatRole::User,
        content: observation,
    });
    if let Some(last) = session.messages.last() {
        transcript.push_str(&last.content);
        transcript.push('\n');
    }
    Ok(())
}

pub(crate) fn exhausted_error(max_bash_turns: u32, transcript: &str) -> AgentError {
    AgentError(format!(
        "bash_loop: exhausted --mini-max-bash-turns ({max_bash_turns}) with partial transcript:\n{transcript}"
    ))
}
