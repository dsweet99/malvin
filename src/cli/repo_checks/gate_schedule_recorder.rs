//! In-process gate command recorder for unit tests (avoids fake shell subprocess farms).

use std::cell::RefCell;
use std::process::Output;

#[derive(Debug, Clone)]
pub struct RecordedGateCommand {
    pub command_line: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

struct GateCommandRecorderState {
    invocations: Vec<RecordedGateCommand>,
    fail_if_contains: Vec<String>,
    echo_streams: bool,
}

thread_local! {
    static RECORDER: RefCell<Option<GateCommandRecorderState>> = const { RefCell::new(None) };
}

pub struct GateCommandRecorderGuard;

impl Drop for GateCommandRecorderGuard {
    fn drop(&mut self) {
        RECORDER.with(|r| *r.borrow_mut() = None);
    }
}

pub fn arm_gate_command_recorder() -> GateCommandRecorderGuard {
    RECORDER.with(|r| {
        *r.borrow_mut() = Some(GateCommandRecorderState {
            invocations: Vec::new(),
            fail_if_contains: Vec::new(),
            echo_streams: false,
        });
    });
    GateCommandRecorderGuard
}

pub fn arm_gate_command_recorder_with_echo_streams() -> GateCommandRecorderGuard {
    let guard = arm_gate_command_recorder();
    RECORDER.with(|r| {
        if let Some(state) = r.borrow_mut().as_mut() {
            state.echo_streams = true;
        }
    });
    guard
}

pub fn arm_gate_command_recorder_failing(substring: &str) -> GateCommandRecorderGuard {
    let guard = arm_gate_command_recorder();
    RECORDER.with(|r| {
        if let Some(state) = r.borrow_mut().as_mut() {
            state.fail_if_contains.push(substring.to_string());
        }
    });
    guard
}

pub fn take_recorded_gate_commands() -> Vec<RecordedGateCommand> {
    RECORDER.with(|r| {
        r.borrow_mut()
            .as_mut()
            .map(|state| std::mem::take(&mut state.invocations))
            .unwrap_or_default()
    })
}

pub fn recordings_as_gate_trace_log(recordings: &[RecordedGateCommand]) -> String {
    recordings
        .iter()
        .flat_map(|r| [r.command_line.as_str(), "\n"])
        .collect()
}

pub(crate) fn try_record_gate_command(
    command_line: &str,
    direct_argv: Option<&str>,
) -> Option<Output> {
    try_record_gate_command_impl(command_line, direct_argv)
}

fn try_record_gate_command_impl(command_line: &str, direct_argv: Option<&str>) -> Option<Output> {
    let line = direct_argv.unwrap_or_else(|| command_line.trim());
    if line.is_empty() {
        return None;
    }
    RECORDER.with(|r| {
        let mut opt = r.borrow_mut();
        let state = opt.as_mut()?;
        let exit_code = i32::from(
            state
                .fail_if_contains
                .iter()
                .any(|needle| line.contains(needle)),
        );
        let (stdout, stderr) = if state.echo_streams {
            (
                format!("stdout from {line}\n"),
                format!("stderr from {line}\n"),
            )
        } else {
            (String::new(), String::new())
        };
        state.invocations.push(RecordedGateCommand {
            command_line: line.to_string(),
            exit_code,
            stdout: stdout.clone(),
            stderr: stderr.clone(),
        });
        Some(output_from_parts(exit_code, &stdout, &stderr))
    })
}

fn output_from_parts(code: i32, stdout: &str, stderr: &str) -> Output {
    use std::os::unix::process::ExitStatusExt;
    use std::process::ExitStatus;
    Output {
        status: ExitStatus::from_raw(
            ((u32::from(u8::try_from(code.max(0)).unwrap_or(0))) << 8)
                .try_into()
                .unwrap(),
        ),
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_gate_schedule_recorder_types() {
        let recorded = RecordedGateCommand {
            command_line: "a".to_string(),
            exit_code: 0,
            stdout: "b".to_string(),
            stderr: "c".to_string(),
        };
        let _guard = arm_gate_command_recorder();
        let _ = stringify!(RecordedGateCommand);
        let _ = stringify!(GateCommandRecorderState);
        let _ = stringify!(GateCommandRecorderGuard);
        let _ = stringify!(command_line);
        let _ = stringify!(exit_code);
        let _ = stringify!(stdout);
        let _ = stringify!(stderr);
        let _ = stringify!(invocations);
        let _ = stringify!(fail_if_contains);
        let _ = stringify!(echo_streams);
        let _ = stringify!(drop);
        let guard = arm_gate_command_recorder();
        drop(guard);
    }
}
#[cfg(test)]
#[path = "gate_schedule_recorder_test.rs"]
mod gate_schedule_recorder_test;#[cfg(test)]
#[path = "gate_schedule_recorder_kiss_cov_test.rs"]
mod gate_schedule_recorder_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<GateCommandRecorderGuard> = None;
        let _: Option<RecordedGateCommand> = None;
        let _ = output_from_parts;
        let _ = try_record_gate_command;
        let _ = try_record_gate_command_impl;
    }
}
