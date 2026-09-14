use crate::cli::admin_cmd::AdminArgs;
use crate::cli::args::{Cli, Commands};
use crate::cli::shared_opts::{RouterOpts, SharedOpts};
use malvin::model_id::ParsedModel;

#[derive(Debug)]
pub(crate) enum MalvinWorkflow {
    Do {
        request: Option<String>,
        shared: SharedOpts,
    },
    Admin {
        admin: AdminArgs,
        model: ParsedModel,
    },
    DefaultRoute {
        request: String,
        shared: SharedOpts,
        router: RouterOpts,
    },
    GatesOnly {
        shared: SharedOpts,
        router: RouterOpts,
    },
}

#[must_use]
pub(crate) fn malvin_workflow_from_cli(cli: Cli) -> Option<MalvinWorkflow> {
    if cli.do_workflow {
        return Some(MalvinWorkflow::Do {
            request: cli.request,
            shared: cli.shared,
        });
    }
    if let Some(Commands::Admin(admin)) = cli.command {
        return Some(MalvinWorkflow::Admin {
            admin,
            model: cli.shared.model,
        });
    }
    if let Some(request) = cli.request {
        return Some(MalvinWorkflow::DefaultRoute {
            request,
            shared: cli.shared,
            router: cli.router,
        });
    }
    if cli.router.gates {
        return Some(MalvinWorkflow::GatesOnly {
            shared: cli.shared,
            router: cli.router,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{MalvinWorkflow, malvin_workflow_from_cli};
    use crate::cli::Cli;
    use clap::Parser;

    #[test]
    fn do_workflow_omits_loop_budgets_and_router_opts_from_payload() {
        let cli = Cli::try_parse_from(["malvin", "--do", "fix it"]).expect("parse");
        let workflow = malvin_workflow_from_cli(cli).expect("do workflow");
        match workflow {
            MalvinWorkflow::Do { request, shared: _ } => {
                assert_eq!(request.as_deref(), Some("fix it"));
            }
            MalvinWorkflow::Admin { .. }
            | MalvinWorkflow::DefaultRoute { .. }
            | MalvinWorkflow::GatesOnly { .. } => {
                panic!("expected Do variant without loop-budget or router fields")
            }
        }
    }

    #[test]
    fn do_workflow_payload_has_no_router_fields() {
        // Compile-time shape check: Do carries SharedOpts only (no RouterOpts / quiet).
        let cli = Cli::try_parse_from(["malvin", "--do", "x"]).expect("parse");
        let workflow = malvin_workflow_from_cli(cli).expect("do");
        let MalvinWorkflow::Do { shared, .. } = workflow else {
            panic!("expected Do");
        };
        let _ = shared.model;
        let _ = shared.verbose;
        let _ = shared.max_acp_retries;
        // quiet lives on RouterOpts; SharedOpts has no quiet field after the peel.
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
                request,
                router,
                ..
            } => {
                assert_eq!(request, "build it");
                assert_eq!(router.max_loops, 4);
                assert_eq!(router.max_hypotheses, 7);
                assert_eq!(router.creative, Some(0.5));
                assert!(!router.gates);
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
        let cli = Cli::try_parse_from([
            "malvin",
            "--model",
            "cursor:auto",
            "admin",
            "models",
        ])
        .expect("parse");
        let workflow = malvin_workflow_from_cli(cli).expect("admin workflow");
        match workflow {
            MalvinWorkflow::Admin { model, .. } => {
                assert_eq!(model.canonical(), "cursor:auto");
            }
            other => panic!("expected Admin, got {other:?}"),
        }
    }
}
