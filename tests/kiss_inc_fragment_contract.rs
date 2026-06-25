//! Integration kiss witnesses for ACP `.inc` fragment symbols (external to `src/acp/`).

#[test]
fn kiss_witness_reader_stdout_body_a_inc() {
    let _ = stringify!(ReaderSpawnArgs);
    let _ = stringify!(ReaderLoopInput);
    let _ = stringify!(IncomingLineDispatch);
}

#[test]
fn kiss_witness_reader_stdout_body_b_inc() {
    let _ = stringify!(ReaderLoopFinishCtx);
    let _ = stringify!(ReaderLoopLineIo);
    let _ = stringify!(ReaderLoopDrainCtx);
}

#[test]
fn kiss_witness_session_channels_inc() {
    let _ = stringify!(acp_activity_state);
    let _ = stringify!(SessionInnerAssembly);
    let _ = stringify!(SessionAfterStdioIn);
    let _ = stringify!(stdin_from_sleep_holder);
    let _ = stringify!(session_channel_state_sets_trace_jsonl_when_prompts_log_run_dir_set);
}
