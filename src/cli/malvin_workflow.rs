use crate::cli::admin_cmd::AdminArgs;
use crate::cli::args::{Cli, Commands};
use crate::cli::request_argv::{RequestKind, TaggedRequest};
use crate::cli::shared_opts::{RouterOpts, SharedOpts};
use malvin::model_id::ParsedModel;

#[derive(Debug)]
pub(crate) enum MalvinWorkflow {
    Do {
        requests: Vec<String>,
        shared: SharedOpts,
    },
    Admin {
        admin: AdminArgs,
        model: ParsedModel,
    },
    DefaultRoute {
        jobs: Vec<TaggedRequest>,
        shared: SharedOpts,
        router: RouterOpts,
    },
    Mixed {
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
    let kind = if cli.do_workflow() {
        RequestKind::Do
    } else {
        RequestKind::Router
    };
    let creative = if matches!(kind, RequestKind::Router) {
        cli.router.creative_probability()
    } else {
        None
    };
    cli.requests
        .iter()
        .map(|text| TaggedRequest {
            text: text.clone(),
            kind,
            creative,
        })
        .collect()
}

fn texts_of_kind(jobs: &[TaggedRequest], kind: RequestKind) -> Vec<String> {
    jobs.iter()
        .filter(|j| j.kind == kind)
        .map(|j| j.text.clone())
        .collect()
}

fn router_jobs(jobs: Vec<TaggedRequest>) -> Vec<TaggedRequest> {
    jobs.into_iter()
        .filter(TaggedRequest::is_router)
        .collect()
}

#[must_use]
pub(crate) fn malvin_workflow_from_cli(cli: Cli) -> Option<MalvinWorkflow> {
    if let Some(Commands::Admin(admin)) = cli.command {
        return Some(MalvinWorkflow::Admin {
            admin,
            model: cli.shared.model,
        });
    }
    let jobs = synthesize_tagged(&cli);
    if !jobs.is_empty() {
        let any_do = jobs.iter().any(TaggedRequest::is_do);
        let any_router = jobs.iter().any(TaggedRequest::is_router);
        if any_do && any_router {
            return Some(MalvinWorkflow::Mixed {
                jobs,
                shared: cli.shared,
                router: cli.router,
            });
        }
        if any_do {
            return Some(MalvinWorkflow::Do {
                requests: texts_of_kind(&jobs, RequestKind::Do),
                shared: cli.shared,
            });
        }
        return Some(MalvinWorkflow::DefaultRoute {
            jobs: router_jobs(jobs),
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
