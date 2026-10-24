use super::{Completion, Iteration, RunOutcome};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RouterState {
    Starting(Iteration),
    AwaitingAudit(Iteration),
    AwaitingWork(Iteration),
    AwaitingCompletion(Iteration),
    Restoring {
        iteration: Iteration,
        completion: Completion,
    },
    AwaitingGates(Iteration),
    Summarizing {
        outcome: RunOutcome,
    },
    Closing {
        next: Option<Iteration>,
        outcome: RunOutcome,
    },
    Finished(RunOutcome),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Effect {
    StartIterationSession,
    SendAudit,
    SendWork,
    SendCompletionCheck,
    RestoreSessionFiles,
    RunGates,
    SendSummary,
    CloseSession,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the driver currently enters the reducer after file restoration"
    )
)]
pub(crate) enum RouterEvent {
    SessionStarted,
    AuditFoundImpasse,
    AuditNeedsWork,
    WorkCompleted,
    CompletionChecked { satisfied: bool },
    FilesRestored { gates_enabled: bool },
    GatesPassed,
    GatesFailed,
    SummarySent,
    SessionClosed,
    EffectFailed,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Transition {
    pub(crate) next: RouterState,
    pub(crate) effect: Option<Effect>,
}
