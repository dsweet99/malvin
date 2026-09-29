use crate::acp::AgentError;
use crate::bridge_protocol::BridgeEvent;
use crate::bridge_sdk::{StreamLog, record_sdk_usage, run_done_status_is_failure};

pub(super) fn finish_output_cap(
    log: &StreamLog,
    ev: &mut BridgeEvent,
    message: String,
) -> AgentError {
    if let BridgeEvent::RunDone { status, error, .. } = ev {
        *status = crate::bridge_protocol::RunDoneStatus::Error;
        *error = Some(message.clone());
    }
    let _ = finish_run_done(log, ev);
    AgentError {
        message,
        fault: crate::acp::AgentFault::OutputCap,
    }
}

pub(crate) fn finish_run_done(log: &StreamLog, ev: &BridgeEvent) -> Result<(), AgentError> {
    let BridgeEvent::RunDone {
        status,
        result,
        usage,
        error,
        ..
    } = ev
    else {
        return Ok(());
    };
    if let Some(u) = usage {
        record_sdk_usage(log.timing.as_ref(), u);
    }
    *log.last_response
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = result.clone().unwrap_or_default();
    if let Some(text) = result {
        crate::bridge_sdk::feed_do_dm_run_result(text);
    }
    crate::bridge_sdk::handle_stream_event(log, ev);
    if *status == crate::bridge_protocol::RunDoneStatus::Unknown {
        tracing::warn!(
            result = result.as_deref(),
            error = error.as_deref(),
            "run_done unknown status; surfacing result/error"
        );
    }
    if run_done_status_is_failure(*status) {
        return Err(AgentError(error.clone().unwrap_or_else(|| match *status {
            crate::bridge_protocol::RunDoneStatus::Cancelled => "run cancelled".into(),
            crate::bridge_protocol::RunDoneStatus::Unknown => {
                "run finished with unknown status".into()
            }
            _ => "run error".into(),
        })));
    }
    Ok(())
}
