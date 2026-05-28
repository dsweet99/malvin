use crate::cli::args::{Cli, Commands};
use crate::cli::loop_opts::{apply_tenacious, TENACIOUS_MAX_ACP_RETRIES, TENACIOUS_MAX_LOOPS};
use clap::Parser;

#[test]
fn kpop_tenacious_expands_max_loops_and_acp_retries() {
    let cli = Cli::try_parse_from(["malvin", "kpop", "--tenacious", "investigate"]).expect("parse");
    let Some(Commands::Kpop(mut kpop)) = cli.command else {
        panic!("expected kpop subcommand");
    };
    let mut shared = cli.shared;
    apply_tenacious(
        &mut kpop.max_loops,
        &mut shared.max_acp_retries,
        kpop.tenacious,
    );
    assert_eq!(kpop.max_loops, TENACIOUS_MAX_LOOPS);
    assert_eq!(shared.max_acp_retries, TENACIOUS_MAX_ACP_RETRIES);
}
