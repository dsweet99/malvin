#[test]
fn kiss_witness_do_run_prep() {
    let _: Option<super::DoRunPrep> = None;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(session_dotfile_backups);
    let _ = super::new_do_client;
    let _ = super::prepare_do_run;
    let _ = super::begin_do_session_overlapping_prompt_prep;
    let _ = crate::cli::one_shot_session::resolve_one_shot_request_artifacts;
}
