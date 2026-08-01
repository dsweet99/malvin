//! Bash observation as New request for the inner bash-fence loop.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::agent_backend::mini::bash_adapter::{format_observation, run_bash_command, BashExecResult};
use crate::agent_backend::mini::fence_parser::BashFence;
use super::loop_inner_types::LoopCounters;
use super::loop_types::LoopDriverSession;
use crate::acp::AgentError;
use crate::run_timing::RunTiming;
use crate::tool_summary::{bash_kind_wire_name, classify_bash_command};

pub(crate) struct BashObservationInput<'a> {
    pub(crate) session: &'a mut LoopDriverSession,
    pub(crate) trace: &'a crate::agent_backend::mini::trace::MiniTraceSink,
    pub(crate) transcript: &'a mut String,
    pub(crate) counters: &'a mut LoopCounters,
    pub(crate) timing: Option<&'a Arc<Mutex<RunTiming>>>,
}

fn record_bash_tool_wall(timing: Option<&Arc<Mutex<RunTiming>>>, command: &str, elapsed: Duration) {
    let Some(timing) = timing else {
        return;
    };
    let kind = bash_kind_wire_name(classify_bash_command(command));
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .add_tool_call_wall(kind, elapsed);
}

fn run_one_bash_fence(
    input: &mut BashObservationInput<'_>,
    fence: &BashFence,
) -> Result<BashExecResult, AgentError> {
    crate::agent_phase::note_mini_bash_exec();
    input.session.bash_commands_this_prompt.push(fence.command.clone());
    let t0 = Instant::now();
    let result = run_bash_command(
        input.session.cwd.as_path(),
        &fence.command,
        &input.session.llm_model_slug,
    )
    .map_err(AgentError)?;
    input.counters.bash_exec_count += 1;
    let elapsed = t0.elapsed();
    record_bash_tool_wall(input.timing, &fence.command, elapsed);
    input
        .trace
        .mini_bash_exec(&fence.command, result.exit_code, elapsed, fence.comment.as_deref());
    crate::agent_phase::note_mini_bash_exec_done(result.exit_code, &fence.command);
    Ok(result)
}

pub(crate) fn append_bash_observation(
    fences: &[BashFence],
    mut input: BashObservationInput<'_>,
) -> Result<(), AgentError> {
    let mut results: Vec<BashExecResult> = Vec::new();
    for fence in fences {
        results.push(run_one_bash_fence(&mut input, fence)?);
    }
    let observation = format_observation(&results);
    input.transcript.push_str(&observation);
    input.transcript.push('\n');
    input.session.pending_new_request = Some(observation);
    input.session.section_shape_nudged = false;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_backend::mini::fence_parser::parse_bash_fences;
    use crate::agent_backend::mini::trace::MiniTraceSink;
    use crate::agent_backend::test_support::test_io;
    use std::path::PathBuf;

    #[test]
    fn append_bash_observation_records_tool_call_wall_time() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let timing = crate::run_timing::RunTiming::new_arc();
        let mut session = LoopDriverSession {
            history: String::new(),
            previous_response: String::new(),
            pending_new_request: None,
            cwd: tmp.path().to_path_buf(),
            bash_commands_this_prompt: vec![],
            prompt_index: 0,
            llm_model_slug: "test".into(),
            section_shape_nudged: false,
        };
        let trace = MiniTraceSink::new(Some(PathBuf::from(tmp.path())), test_io());
        let mut transcript = String::new();
        let mut counters = LoopCounters {
            http_turn_count: 0,
            bash_exec_count: 0,
            investigate_http_turns: 0,
            had_bash_this_prompt: false,
        };
        let fences = parse_bash_fences("```bash\nsleep 0.05\n```");
        append_bash_observation(
            &fences,
            BashObservationInput {
                session: &mut session,
                trace: &trace,
                transcript: &mut transcript,
                counters: &mut counters,
                timing: Some(&timing),
            },
        )
        .expect("bash");
        let ms = {
            let tmp2 = tempfile::tempdir().unwrap();
            timing.lock().unwrap().write_json_only(tmp2.path()).unwrap();
            let v: serde_json::Value = serde_json::from_slice(
                &std::fs::read(tmp2.path().join(crate::run_timing::RUN_TIMING_JSON_FILE)).unwrap(),
            )
            .unwrap();
            v["tool_calls_ms"].as_u64().unwrap()
        };
        assert!(
            ms >= 40,
            "mini bash must record tool_calls_ms (>=40ms for sleep 0.05), got {ms}"
        );
        assert_eq!(counters.bash_exec_count, 1);
    }
}
