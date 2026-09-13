#[path = "tidy_flow/run.rs"]
mod run;

pub use run::run_tidy;

#[cfg(test)]
pub(crate) use run::{TIDY_ROUTER_REQUEST, tidy_router_with_gates_forced};

#[must_use]
pub(crate) fn effective_tidy_max_loops(max_loops: usize) -> usize {
    crate::cli::workflow_router_shared::effective_max_loops(max_loops)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{RouterOpts, SharedOpts};

    #[test]
    fn tidy_effective_max_loops_is_at_least_one() {
        assert_eq!(effective_tidy_max_loops(0), 1);
    }

    #[test]
    fn tidy_router_request_is_get_the_gates_to_pass() {
        assert_eq!(TIDY_ROUTER_REQUEST, "Get the gates to pass.");
    }

    #[test]
    fn tidy_forces_gates_on_regardless_of_cli() {
        let shared = SharedOpts::test_defaults();
        let router = RouterOpts::test_defaults();
        assert!(!router.gates);
        let forced = tidy_router_with_gates_forced(&router);
        assert!(forced.gates);
        let _ = shared;
    }
}
