#[test]
fn prompt_stdout_replacement_maps_learn_placeholder() {
    assert_eq!(crate::acp::prompt_stdout_replacement(crate::output::MALVIN_WHO), None);
    assert_eq!(
        crate::acp::prompt_stdout_replacement("learn"),
        Some(crate::malvin_constants::LEARNING_PLACEHOLDER)
    );
}
