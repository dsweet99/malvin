use crate::cli::admin_cmd::AdminArgs;
use crate::cli::args::{Cli, Commands};
use crate::cli::request_argv::{RequestKind, TaggedRequest};
use crate::cli::shared_opts::{RouterOpts, SharedOpts};
use malvin::config::model_id::ParsedModel;

#[derive(Debug)]
pub(crate) enum MalvinWorkflow {
    Admin {
        admin: AdminArgs,
        model: ParsedModel,
    },
    DefaultRoute {
        jobs: Vec<TaggedRequest>,
        shared: SharedOpts,
        router: RouterOpts,
    },
    GatesOnly {
        shared: SharedOpts,
        router: RouterOpts,
    },
}

fn synthesize_tagged(cli: &Cli) -> Vec<TaggedRequest> {
    if !cli.tagged_requests.is_empty() {
        return cli.tagged_requests.clone();
    }
    cli.requests
        .iter()
        .map(|text| TaggedRequest {
            text: text.clone(),
            kind: RequestKind::Router,
        })
        .collect()
}

#[must_use]
pub(crate) fn malvin_workflow_from_cli(cli: Cli) -> Option<MalvinWorkflow> {
    if let Some(Commands::Admin(admin)) = cli.command {
        return Some(MalvinWorkflow::Admin {
            admin,
            model: cli.shared.model.first().clone(),
        });
    }
    let jobs = synthesize_tagged(&cli);
    if !jobs.is_empty() {
        return Some(MalvinWorkflow::DefaultRoute {
            jobs,
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
#[path = "malvin_workflow_tests.rs"]
mod tests;
