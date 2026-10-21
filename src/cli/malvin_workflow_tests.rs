use super::{MalvinWorkflow, malvin_workflow_from_cli};
use crate::cli::Cli;
use crate::cli::config_defaults::parse_cli_with_config_defaults;
use clap::Parser;
use malvin::test_support::test_utils::with_isolated_home;

fn parse(argv: &[&str]) -> Cli {
    let mut out = None;
    with_isolated_home(|_work| {
        out = Some(parse_cli_with_config_defaults(argv).expect("parse").0);
    });
    out.expect("parsed")
}

#[test]
fn default_route_carries_quiet_on_router_opts() {
    let cli = Cli::try_parse_from(["malvin", "-q", "build it"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute { router, .. } => {
            assert!(router.dm_stdout_only());
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
}

#[test]
fn default_route_carries_max_loops() {
    let cli = Cli::try_parse_from(["malvin", "--max-loops", "7", "build it"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute { router, .. } => {
            assert_eq!(router.max_loops, 7);
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
    let gates = parse(&["malvin", "--max-loops", "6", "-g"]);
    let workflow = malvin_workflow_from_cli(gates).expect("gates only");
    match workflow {
        MalvinWorkflow::GatesOnly { router, .. } => {
            assert_eq!(router.max_loops, 6);
        }
        other => panic!("expected GatesOnly, got {other:?}"),
    }
}

#[test]
fn default_route_carries_router_opts() {
    let cli = Cli::try_parse_from(["malvin", "build it"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("default route");
    match workflow {
        MalvinWorkflow::DefaultRoute { jobs, router, .. } => {
            assert_eq!(jobs.len(), 1);
            assert_eq!(jobs[0].text, "build it");
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
        MalvinWorkflow::DefaultRoute { jobs, .. } => {
            assert_eq!(
                jobs.iter().map(|j| j.text.as_str()).collect::<Vec<_>>(),
                vec!["plan_1.md", "plan_2.md"]
            );
        }
        other => panic!("expected DefaultRoute, got {other:?}"),
    }
}

#[test]
fn gates_only_carries_router_opts_without_request() {
    let cli = Cli::try_parse_from(["malvin", "-g"]).expect("parse");
    let workflow = malvin_workflow_from_cli(cli).expect("gates only");
    match workflow {
        MalvinWorkflow::GatesOnly { router, .. } => {
            assert!(router.gates);
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
