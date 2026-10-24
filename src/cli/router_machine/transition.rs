use super::{Completion, Effect, RouterEvent, RouterState, RunOutcome, Transition};

pub(crate) fn transition(
    state: RouterState,
    event: RouterEvent,
) -> Result<Transition, super::ProtocolError> {
    if matches!(event, RouterEvent::EffectFailed) {
        effect_failed(state)
    } else {
        dispatch(state, event)
    }
}

fn dispatch(state: RouterState, event: RouterEvent) -> Result<Transition, super::ProtocolError> {
    match state {
        RouterState::Starting(i) => started(i, event),
        RouterState::AwaitingAudit(i) => audit(i, event),
        RouterState::AwaitingWork(i) => work(i, event),
        RouterState::AwaitingCompletion(i) => completion(i, event),
        RouterState::Restoring {
            iteration,
            completion,
        } => restored(iteration, completion, event),
        RouterState::AwaitingGates(i) => gates(i, event),
        RouterState::Summarizing { outcome } => summary(outcome, event),
        RouterState::Closing { next: n, outcome } => closing(n, outcome, event),
        RouterState::Finished(outcome) => invalid(RouterState::Finished(outcome), event),
    }
}

const fn effect_failed(state: RouterState) -> Result<Transition, super::ProtocolError> {
    match state {
        RouterState::Finished(outcome) => {
            invalid(RouterState::Finished(outcome), RouterEvent::EffectFailed)
        }
        RouterState::Closing { outcome, .. } => Ok(Transition {
            next: RouterState::Finished(if matches!(outcome, RunOutcome::GatesFailed) {
                RunOutcome::GatesFailed
            } else {
                RunOutcome::AgentFailed
            }),
            effect: None,
        }),
        RouterState::Summarizing { outcome } => close(None, failure_outcome(outcome)),
        _ => close(None, RunOutcome::AgentFailed),
    }
}

const fn failure_outcome(outcome: RunOutcome) -> RunOutcome {
    if matches!(outcome, RunOutcome::GatesFailed) {
        RunOutcome::GatesFailed
    } else {
        RunOutcome::AgentFailed
    }
}
const fn started(i: super::Iteration, e: RouterEvent) -> Result<Transition, super::ProtocolError> {
    if matches!(e, RouterEvent::SessionStarted) {
        next(RouterState::AwaitingAudit(i), Effect::SendAudit)
    } else {
        invalid(RouterState::Starting(i), e)
    }
}
const fn audit(i: super::Iteration, e: RouterEvent) -> Result<Transition, super::ProtocolError> {
    match e {
        RouterEvent::AuditFoundImpasse => restore(i, Completion::Impasse),
        RouterEvent::AuditNeedsWork => next(RouterState::AwaitingWork(i), Effect::SendWork),
        _ => invalid(RouterState::AwaitingAudit(i), e),
    }
}
const fn work(i: super::Iteration, e: RouterEvent) -> Result<Transition, super::ProtocolError> {
    if matches!(e, RouterEvent::WorkCompleted) {
        next(
            RouterState::AwaitingCompletion(i),
            Effect::SendCompletionCheck,
        )
    } else {
        invalid(RouterState::AwaitingWork(i), e)
    }
}
const fn completion(
    i: super::Iteration,
    e: RouterEvent,
) -> Result<Transition, super::ProtocolError> {
    if let RouterEvent::CompletionChecked { satisfied } = e {
        restore(
            i,
            if satisfied {
                Completion::Satisfied
            } else {
                Completion::StillOpen
            },
        )
    } else {
        invalid(RouterState::AwaitingCompletion(i), e)
    }
}
fn restored(
    i: super::Iteration,
    c: Completion,
    e: RouterEvent,
) -> Result<Transition, super::ProtocolError> {
    if let RouterEvent::FilesRestored { gates_enabled } = e {
        after_restore(i, c, gates_enabled)
    } else {
        invalid(
            RouterState::Restoring {
                iteration: i,
                completion: c,
            },
            e,
        )
    }
}
fn gates(i: super::Iteration, e: RouterEvent) -> Result<Transition, super::ProtocolError> {
    match e {
        RouterEvent::GatesPassed => summarize(RunOutcome::Succeeded),
        RouterEvent::GatesFailed => after_gates(i),
        _ => invalid(RouterState::AwaitingGates(i), e),
    }
}
const fn summary(o: RunOutcome, e: RouterEvent) -> Result<Transition, super::ProtocolError> {
    if matches!(e, RouterEvent::SummarySent) {
        close(None, o)
    } else {
        invalid(RouterState::Summarizing { outcome: o }, e)
    }
}
fn closing(
    n: Option<super::Iteration>,
    o: RunOutcome,
    e: RouterEvent,
) -> Result<Transition, super::ProtocolError> {
    if !matches!(e, RouterEvent::SessionClosed) {
        return invalid(
            RouterState::Closing {
                next: n,
                outcome: o,
            },
            e,
        );
    }
    n.map_or(
        Ok(Transition {
            next: RouterState::Finished(o),
            effect: None,
        }),
        |i| next(RouterState::Starting(i), Effect::StartIterationSession),
    )
}
const fn invalid(
    state: RouterState,
    event: RouterEvent,
) -> Result<Transition, super::ProtocolError> {
    Err(super::ProtocolError::InvalidEvent { state, event })
}
#[expect(
    clippy::unnecessary_wraps,
    reason = "transition helpers share one result type"
)]
const fn next(next: RouterState, effect: Effect) -> Result<Transition, super::ProtocolError> {
    Ok(Transition {
        next,
        effect: Some(effect),
    })
}
const fn restore(i: super::Iteration, c: Completion) -> Result<Transition, super::ProtocolError> {
    next(
        RouterState::Restoring {
            iteration: i,
            completion: c,
        },
        Effect::RestoreSessionFiles,
    )
}
const fn summarize(o: RunOutcome) -> Result<Transition, super::ProtocolError> {
    next(RouterState::Summarizing { outcome: o }, Effect::SendSummary)
}
const fn close(
    n: Option<super::Iteration>,
    o: RunOutcome,
) -> Result<Transition, super::ProtocolError> {
    next(
        RouterState::Closing {
            next: n,
            outcome: o,
        },
        Effect::CloseSession,
    )
}
fn after_restore(
    i: super::Iteration,
    c: Completion,
    gates: bool,
) -> Result<Transition, super::ProtocolError> {
    match c {
        Completion::Impasse | Completion::Satisfied if gates => {
            next(RouterState::AwaitingGates(i), Effect::RunGates)
        }
        Completion::StillOpen if !i.is_last() => close(Some(i.next()), RunOutcome::Succeeded),
        _ => summarize(RunOutcome::Succeeded),
    }
}
fn after_gates(i: super::Iteration) -> Result<Transition, super::ProtocolError> {
    if i.is_last() {
        summarize(RunOutcome::GatesFailed)
    } else {
        close(Some(i.next()), RunOutcome::Succeeded)
    }
}
