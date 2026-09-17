use super::{MalvinWorkflow, malvin_workflow_from_cli};
use crate::cli::Cli;
use crate::cli::config_defaults::parse_cli_with_config_defaults;
use crate::cli::request_argv::RequestKind;
use clap::Parser;
use malvin::test_utils::with_isolated_home;

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work| {
        out = Some(parse_cli_with_config_defaults(argv).expect("parse").0);
    });
    out.expect("parsed")
}

#[test]
fn do_workflow_omits_loop_budgets_and_router_opts_from_payload() {
    let cli = Cli::try_parse_from(["malvin", "--do", "fix it"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("do workflow");
    match workflow {
        MalvinWorkflow::Do {
            requests,
            shared: _,
        } => {
            assert_eq!(requests, vec!["fix it".to_string()]);
        }
        MalvinWorkflow::Admin { .. }
        | MalvinWorkflow::DefaultRoute { .. }
        | MalvinWorkflow::Mixed { .. }
        | MalvinWorkflow::GatesOnly { .. } => {
            panic!("expected Do variant without loop-budget or router fields")
        }
    }
}

#[test]
fn do_workflow_payload_has_no_router_fields() {
    let cli = Cli::try_parse_from(["malvin", "--do", "x"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("do");
    let MalvinWorkflow::Do { shared, .. } = workflow else {
        panic!("expected Do");
    };
    let _ = shared.model;
    let _ = shared.verbose;
    let _ = shared.max_acp_retries;
}

#[test]
fn default_route_carries_quiet_on_router_opts() {
    let cli = Cli::try_parse_from(["malvin", "-q", "build it"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute { router, .. } => {
            assert!(router.quiet);
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
}

#[test]
fn default_route_carries_loop_budgets_and_router_opts() {
    let cli = Cli::try_parse_from([
        "malvin",
        "--max-loops",
        "4",
        "--max-hypotheses",
        "7",
        "--creative=0.5",
        "build it",
    ])
    .expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute {
            requests, router, ..
        } => {
            assert_eq!(requests, vec!["build it".to_string()]);
            assert_eq!(router.max_loops, 4);
            assert_eq!(router.max_hypotheses, 7);
            assert_eq!(router.creative, Some(0.5));
            assert!(!router.gates);
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
}

#[test]
fn default_route_carries_multiple_independent_requests() {
    let cli = Cli::try_parse_from(["malvin", "plan_1.md", "plan_2.md"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute { requests, .. } => {
            assert_eq!(
                requests,
                vec!["plan_1.md".to_string(), "plan_2.md".to_string()]
            );
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
}

#[test]
fn gates_only_carries_loop_budgets_without_request() {
    let cli = Cli::try_parse_from(["malvin", "-g", "--max-loops", "2"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("gates only");
    match workflow {
        MalvinWorkflow::GatesOnly { router, .. } => {
            assert!(router.gates);
            assert_eq!(router.max_loops, 2);
        }
        other => panic!("expected GatesOnly, got {other:?}"),
    }
}

#[test]
fn empty_cli_maps_to_none() {
    let cli = Cli::try_parse_from(["malvin"]).expect("parse");
    assert!(malvin_workflow_from_cli(cli).is_none());
}

#[test]
fn admin_workflow_payload_carries_model_only() {
    let cli = Cli::try_parse_from(["malvin", "--model", "cursor:auto", "admin", "models"])
        .expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("admin workflow");
    match workflow {
        MalvinWorkflow::Admin { model, .. } => {
            assert_eq!(model.canonical(), "cursor:auto");
        }
        other => panic!("expected Admin, got {other:?}"),
    }
}

#[test]
fn mixed_do_and_router_requests_preserve_order() {
    let cli = parse(&[
        "malvin",
        "Write",
        "--do",
        "What time",
        "Find bug",
        "--do",
        "Summarize",
    ]);
    let workflow = malvin_workflow_from_cli(cli).expect("mixed");
    match workflow {
        MalvinWorkflow::Mixed { jobs, .. } => {
            assert_eq!(
                jobs.iter()
                    .map(|j| (j.text.as_str(), j.kind))
                    .collect::<Vec<_>>(),
                vec![
                    ("Write", RequestKind::Router),
                    ("What time", RequestKind::Do),
                    ("Find bug", RequestKind::Router),
                    ("Summarize", RequestKind::Do),
                ]
            );
        }
        other => panic!("expected Mixed, got {other:?}"),
    }
}

#[test]
fn do_then_extra_request_is_mixed() {
    let cli = parse(&["malvin", "--do", "Hello", "Research"]);
    let workflow = malvin_workflow_from_cli(cli).expect("mixed");
    match workflow {
        MalvinWorkflow::Mixed { jobs, .. } => {
            assert_eq!(jobs[0].kind, RequestKind::Do);
            assert_eq!(jobs[1].kind, RequestKind::Router);
        }
        other => panic!("expected Mixed, got {other:?}"),
    }
}
