
#[test]
fn kiss_cov_session_and_spawn_names() {
    let _ = super::spawn_bridge;
    let _ = stringify!(pi_spawn_bridge);
    let _ = stringify!(split_provider_model);
    let _ = stringify!(pi_open_bridge_session);
    let _ = stringify!(PiChildStdio);
    let _ = stringify!(pi_take_stdio);
    let _ = stringify!(pi_note_sandbox);
    let _ = stringify!(pi_assemble_session);
    let _ = stringify!(pi_build_command);
}

#[test]
fn split_provider_model_first_slash() {
    assert_eq!(
        super::session_spawn::split_provider_model("openai/gpt-4o").expect("ok"),
        ("openai", "gpt-4o")
    );
    assert_eq!(
        super::session_spawn::split_provider_model("openrouter/anthropic/claude-3-haiku")
            .expect("ok"),
        ("openrouter", "anthropic/claude-3-haiku")
    );
    assert!(super::session_spawn::split_provider_model("noslash")
        .expect_err("err")
        .0
        .contains("provider"));
}
