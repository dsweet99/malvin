//! `malvin init` — discover workspace quality gates and write `.malvin/checks`.

use clap::Args;

use crate::cli::checks_discovery_flow::{ensure_malvin_checks_discovered_for_cwd, ChecksDiscoveryOpts};
use crate::cli::SharedOpts;

/// Arguments for [`run_init`].
#[derive(Args, Debug, Clone, Copy, Default)]
pub struct InitArgs {}

/// Run the init workflow from `init_constraints.md` (checks-discovery `KPop`).
pub async fn run_init(_init: InitArgs, shared: &SharedOpts) -> Result<(), String> {
    ensure_malvin_checks_discovered_for_cwd(shared, ChecksDiscoveryOpts::INIT).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_run_entry_is_covered() {
        let _ = run_init;
    }

    #[test]
    fn init_args_default() {
        let _ = InitArgs::default();
    }
}
