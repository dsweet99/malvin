#![allow(
    clippy::multiple_crate_versions,
    unused_attributes,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::needless_pass_by_value,
    clippy::redundant_pub_crate,
    clippy::unused_async,
    clippy::implicit_hasher,
    clippy::unnecessary_lazy_evaluations,
    clippy::redundant_clone,
    clippy::needless_borrow,
    clippy::elidable_lifetime_names,
    clippy::match_same_arms,
    clippy::ptr_arg,
    clippy::unused_self,
    clippy::assigning_clones,
    clippy::no_effect_underscore_binding,
    clippy::implicit_clone,
    clippy::single_match,
    clippy::needless_pass_by_ref_mut,
    dead_code,
    unused_imports
)]

#[path = "cli/mod.rs"]
mod cli;
#[path = "cli/do_flow.rs"]
mod do_flow;
#[path = "cli/repo_checks/mod.rs"]
mod repo_checks;
#[path = "cli/router_flow.rs"]
mod router_flow;

fn main() -> cli::Exit {
    cli::entrypoint()
}
