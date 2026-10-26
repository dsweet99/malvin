use super::*;
fn one() -> Iteration {
    Iteration::new(1, 1).expect("valid")
}
#[test]
fn transition_table_covers_router_terminal_choices() {
    let impasse = transition(
        RouterState::Restoring {
            iteration: one(),
            completion: Completion::Impasse,
        },
        RouterEvent::FilesRestored {
            gates_enabled: false,
        },
    )
    .expect("transition");
    assert_eq!(impasse.effect, Some(Effect::SendSummary));
    let gated = transition(
        RouterState::Restoring {
            iteration: one(),
            completion: Completion::Satisfied,
        },
        RouterEvent::FilesRestored {
            gates_enabled: true,
        },
    )
    .expect("transition");
    assert_eq!(gated.effect, Some(Effect::RunGates));
    let failed = transition(RouterState::AwaitingGates(one()), RouterEvent::GatesFailed)
        .expect("transition");
    assert_eq!(failed.effect, Some(Effect::SendSummary));
}

#[test]
fn transition_table_covers_the_documented_paths() {
    assert_effect(
        RouterState::Starting(one()),
        RouterEvent::SessionStarted,
        Effect::SendAudit,
    );
    assert_effect(
        RouterState::AwaitingAudit(one()),
        RouterEvent::AuditFoundImpasse,
        Effect::RestoreSessionFiles,
    );
    assert_effect(
        RouterState::AwaitingAudit(one()),
        RouterEvent::AuditNeedsWork,
        Effect::SendWork,
    );
    assert_effect(
        RouterState::AwaitingWork(one()),
        RouterEvent::WorkCompleted,
        Effect::SendCompletionCheck,
    );
    assert_effect(
        RouterState::AwaitingCompletion(one()),
        RouterEvent::CompletionChecked { satisfied: true },
        Effect::RestoreSessionFiles,
    );
    assert_effect(
        RouterState::AwaitingCompletion(one()),
        RouterEvent::CompletionChecked { satisfied: false },
        Effect::RestoreSessionFiles,
    );
    assert_effect(
        RouterState::AwaitingGates(one()),
        RouterEvent::GatesPassed,
        Effect::SendSummary,
    );
    assert_effect(
        RouterState::Summarizing {
            outcome: RunOutcome::Succeeded,
        },
        RouterEvent::SummarySent,
        Effect::CloseSession,
    );
}

fn assert_effect(state: RouterState, event: RouterEvent, effect: Effect) {
    assert_eq!(
        transition(state, event).expect("transition").effect,
        Some(effect)
    );
}
#[test]
fn rejects_events_outside_their_phase() {
    assert!(matches!(
        transition(RouterState::AwaitingAudit(one()), RouterEvent::GatesPassed),
        Err(ProtocolError::InvalidEvent { .. })
    ));
}
#[test]
fn iteration_constructor_excludes_invalid_bounds() {
    assert!(Iteration::new(0, 1).is_err());
    assert!(Iteration::new(2, 1).is_err());
}
#[test]
fn bounded_exploration_preserves_router_invariants() {
    for max in 1..=3 {
        explore(
            RouterState::Starting(Iteration::new(1, max).expect("valid")),
            0,
        );
    }
}

#[test]
fn effect_failures_close_the_session_and_preserve_gate_failure() {
    let failed_prompt = transition(RouterState::AwaitingWork(one()), RouterEvent::EffectFailed)
        .expect("transition");
    assert_eq!(failed_prompt.effect, Some(Effect::CloseSession));
    assert!(matches!(
        failed_prompt.next,
        RouterState::Closing {
            next: None,
            outcome: RunOutcome::AgentFailed,
        }
    ));

    let failed_summary = transition(
        RouterState::Summarizing {
            outcome: RunOutcome::GatesFailed,
        },
        RouterEvent::EffectFailed,
    )
    .expect("transition");
    let closed = transition(failed_summary.next, RouterEvent::EffectFailed).expect("transition");
    assert_eq!(closed.next, RouterState::Finished(RunOutcome::GatesFailed));
}

#[test]
fn every_effect_boundary_has_a_failure_transition() {
    let states = [
        RouterState::Starting(one()),
        RouterState::AwaitingAudit(one()),
        RouterState::AwaitingWork(one()),
        RouterState::AwaitingCompletion(one()),
        RouterState::Restoring {
            iteration: one(),
            completion: Completion::Satisfied,
        },
        RouterState::AwaitingGates(one()),
        RouterState::Summarizing {
            outcome: RunOutcome::Succeeded,
        },
    ];
    for state in states {
        let failure = transition(state, RouterEvent::EffectFailed).expect("failure transition");
        assert_eq!(failure.effect, Some(Effect::CloseSession));
        assert!(matches!(
            failure.next,
            RouterState::Closing { next: None, .. }
        ));
    }

    let close_failure = transition(
        RouterState::Closing {
            next: None,
            outcome: RunOutcome::Succeeded,
        },
        RouterEvent::EffectFailed,
    )
    .expect("close failure transition");
    assert_eq!(
        close_failure.next,
        RouterState::Finished(RunOutcome::AgentFailed)
    );
}

#[test]
fn scripted_executor_records_representative_effect_sequences() {
    let impasse = execute(
        vec![
            RouterEvent::SessionStarted,
            RouterEvent::AuditFoundImpasse,
            RouterEvent::FilesRestored {
                gates_enabled: true,
            },
            RouterEvent::GatesPassed,
            RouterEvent::SummarySent,
            RouterEvent::SessionClosed,
        ],
        1,
    );
    assert_eq!(
        impasse,
        vec![
            Effect::StartIterationSession,
            Effect::SendAudit,
            Effect::RestoreSessionFiles,
            Effect::RunGates,
            Effect::SendSummary,
            Effect::CloseSession,
        ]
    );

    let open_then_satisfied = execute(
        vec![
            RouterEvent::SessionStarted,
            RouterEvent::AuditNeedsWork,
            RouterEvent::WorkCompleted,
            RouterEvent::CompletionChecked { satisfied: false },
            RouterEvent::FilesRestored {
                gates_enabled: false,
            },
            RouterEvent::SessionClosed,
            RouterEvent::SessionStarted,
            RouterEvent::AuditNeedsWork,
            RouterEvent::WorkCompleted,
            RouterEvent::CompletionChecked { satisfied: true },
            RouterEvent::FilesRestored {
                gates_enabled: false,
            },
            RouterEvent::SummarySent,
            RouterEvent::SessionClosed,
        ],
        2,
    );
    assert_eq!(
        open_then_satisfied,
        vec![
            Effect::StartIterationSession,
            Effect::SendAudit,
            Effect::SendWork,
            Effect::SendCompletionCheck,
            Effect::RestoreSessionFiles,
            Effect::CloseSession,
            Effect::StartIterationSession,
            Effect::SendAudit,
            Effect::SendWork,
            Effect::SendCompletionCheck,
            Effect::RestoreSessionFiles,
            Effect::SendSummary,
            Effect::CloseSession,
        ]
    );
}

fn execute(events: Vec<RouterEvent>, max: usize) -> Vec<Effect> {
    let mut state = RouterState::Starting(Iteration::new(1, max).expect("valid"));
    let mut effects = vec![Effect::StartIterationSession];
    for event in events {
        let step = transition(state, event).expect("scripted event is valid");
        state = step.next;
        if let Some(effect) = step.effect {
            effects.push(effect);
        }
    }
    assert!(matches!(state, RouterState::Finished(_)));
    effects
}
fn explore(state: RouterState, summaries: usize) {
    assert!(summaries <= 1);
    let events: &[RouterEvent] = match &state {
        RouterState::Starting(_) => &[RouterEvent::SessionStarted],
        RouterState::AwaitingAudit(_) => {
            &[RouterEvent::AuditFoundImpasse, RouterEvent::AuditNeedsWork]
        }
        RouterState::AwaitingWork(_) => &[RouterEvent::WorkCompleted],
        RouterState::AwaitingCompletion(_) => &[
            RouterEvent::CompletionChecked { satisfied: true },
            RouterEvent::CompletionChecked { satisfied: false },
        ],
        RouterState::Restoring { .. } => &[
            RouterEvent::FilesRestored {
                gates_enabled: false,
            },
            RouterEvent::FilesRestored {
                gates_enabled: true,
            },
        ],
        RouterState::AwaitingGates(_) => &[RouterEvent::GatesPassed, RouterEvent::GatesFailed],
        RouterState::Summarizing { .. } => &[RouterEvent::SummarySent],
        RouterState::Closing { .. } => &[RouterEvent::SessionClosed],
        RouterState::Finished(_) => {
            assert_eq!(summaries, 1, "normal paths summarize exactly once");
            return;
        }
    };
    for event in events {
        let step = transition(state.clone(), *event).expect("reachable");
        if matches!(step.effect, Some(Effect::RunGates)) {
            assert!(matches!(
                &state,
                RouterState::Restoring {
                    completion: Completion::Impasse | Completion::Satisfied,
                    ..
                }
            ));
        }
        if let (
            RouterState::Closing {
                next: Some(planned),
                ..
            },
            RouterEvent::SessionClosed,
            RouterState::Starting(next),
        ) = (&state, event, &step.next)
        {
            assert_eq!(next.current(), planned.current());
        }
        explore(
            step.next,
            summaries + usize::from(matches!(step.effect, Some(Effect::SendSummary))),
        );
    }
}
