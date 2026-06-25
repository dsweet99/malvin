//! External kiss witnesses for `do_flow` private symbols.

#[test]
fn kiss_witness_do_run_prep() {
    let _: Option<super::DoRunPrep> = None;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(session_dotfile_backups);
    let _ = super::new_do_client;
    let _ = super::prepare_do_run;
    let _ = super::run_do_repo_gates_if_requested;
}
