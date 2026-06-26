//! Inner bash-fence loop for one `run_coder_prompt`.

use std::time::Instant;

use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

use crate::agent_backend::mini::bash_adapter::{format_observation, run_bash_command, BashExecResult};
use crate::agent_backend::mini::fence_parser::{
    has_mini_done_outside_bash_fences, parse_bash_fences, BashFence,
};
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

pub(crate) struct TurnContext {
    pub no_fence_nudge_used: bool,
    pub bash_executed: bool,
}

/// Run the inner loop for one user prompt (constraints prepended by caller).
struct TurnDispatch<'a> {
    assistant_text: &'a str,
    session: &'a mut LoopDriverSession,
    trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    transcript: &'a mut String,
    no_fence_nudge_used: &'a mut bool,
    bash_executed: &'a mut bool,
    final_text: &'a mut String,
    max_bash_turns: u32,
    turn: u32,
}

fn apply_turn_action(action: TurnAction, ctx: TurnDispatch<'_>) -> Result<bool, AgentError> {
    let TurnDispatch {
        assistant_text,
        session,
        trace,
        transcript,
        no_fence_nudge_used,
        bash_executed,
        final_text,
        max_bash_turns,
        turn,
    } = ctx;
    match action {
        TurnAction::Done(text) => {
            *final_text = text;
            trace.mini_assistant(final_text);
            Ok(true)
        }
        TurnAction::Continue => {
            trace.stream_assistant_chunks(assistant_text);
            if !*no_fence_nudge_used {
                *no_fence_nudge_used = true;
                session.messages.push(ChatMessage {
                    role: ChatRole::User,
                    content: NO_FENCE_NUDGE.into(),
                });
                trace.log_nudge(NO_FENCE_NUDGE);
            }
            Ok(false)
        }
        TurnAction::RunBash(fences) => {
            trace.stream_assistant_chunks(assistant_text);
            append_bash_observation(session, &fences, trace, transcript)?;
            *bash_executed = true;
            if turn + 1 >= max_bash_turns {
                return Err(exhausted_error(max_bash_turns, transcript));
            }
            Ok(false)
        }
    }
}

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
    let mut bash_executed = false;
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

        let action = classify_turn(&assistant_text, &TurnContext {
            no_fence_nudge_used,
            bash_executed,
        });
        if apply_turn_action(
            action,
            TurnDispatch {
                assistant_text: &assistant_text,
                session,
                trace,
                transcript: &mut transcript,
                no_fence_nudge_used: &mut no_fence_nudge_used,
                bash_executed: &mut bash_executed,
                final_text: &mut final_text,
                max_bash_turns: config.max_bash_turns,
                turn,
            },
        )? {
            break;
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

pub(crate) fn classify_turn(assistant_text: &str, ctx: &TurnContext) -> TurnAction {
    if has_mini_done_outside_bash_fences(assistant_text) {
        return TurnAction::Done(assistant_text.to_string());
    }
    let fences = parse_bash_fences(assistant_text);
    if fences.is_empty() {
        if ctx.no_fence_nudge_used && ctx.bash_executed {
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
        trace.mini_bash_exec(&fence.command, result.exit_code, elapsed, fence.comment.as_deref());
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
