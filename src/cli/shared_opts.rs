//! Shared CLI flags for `code` and `kpop`.

use clap::Args;

#[derive(Args, Debug)]
pub struct SharedOpts {
    /// Model label [default: opus-4.5].
    #[arg(long, default_value = "opus-4.5")]
    pub model: String,
    /// Disable force-mode [default: force on].
    #[arg(long, default_value_t = false)]
    pub no_force: bool,
    /// Do not copy the plan/request to stdout [default: tee on].
    #[arg(long, default_value_t = false)]
    pub no_tee: bool,
}

impl SharedOpts {
    /// Whether to print the primary user document (plan/request) to stdout before the run.
    #[must_use]
    pub(crate) const fn primary_doc_plain_echo(&self) -> bool {
        !self.no_tee
    }
}
