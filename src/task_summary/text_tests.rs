use super::*;

#[test]
fn short_requests_are_clipped_at_a_word_and_marked() {
    assert_eq!(
        local_task("Investigate extraordinarily complicated parsing failures").as_deref(),
        Some("Investigate...")
    );
    assert_eq!(
        local_task("One two three four five six seven eight nine").as_deref(),
        Some("One two three...")
    );
    assert_eq!(
        local_task("Réparer immédiatement les caractères Unicode du résumé").as_deref(),
        Some("Réparer...")
    );
    assert_eq!(
        local_task("Fix the parser now.").as_deref(),
        Some("Fix the parser now.")
    );
}

#[test]
fn summary_is_at_most_twenty_characters() {
    assert_eq!(
        normalize("\"Fix the parser\""),
        Some("Fix the parser".into())
    );
    assert_eq!(
        normalize("Add\ncheap summaries"),
        Some("Add cheap summaries".into())
    );
    assert_eq!(
        normalize("12345678901234567890"),
        Some("12345678901234567890".into())
    );
    assert_eq!(normalize("123456789012345678901"), None);
    assert_eq!(normalize("one two \x1b[31mthree"), None);
    assert_eq!(normalize(" \n"), None);
}

#[test]
fn summary_must_describe_the_source_rather_than_the_directions() {
    let directions = directions("user request");
    assert!(accept("Thanks", "Return only that summary", &directions).is_err());
    assert!(accept("Thanks", "Use 20 characters now", &directions).is_err());
    assert_eq!(
        accept("Thanks", "\"User says thanks\"", &directions).unwrap(),
        "User says thanks"
    );
    assert_eq!(
        accept("Set 20-character cap", "Set 20-character cap", &directions).unwrap(),
        "Set 20-character cap"
    );
}

#[test]
fn stored_direction_text_is_not_publishable() {
    assert_eq!(publishable("Return only that"), None);
    assert_eq!(
        publishable("Add cheap summaries"),
        Some("Add cheap summaries".into())
    );
}

#[test]
fn model_summaries_fit_at_word_and_unicode_boundaries() {
    let directions = directions("agent's current status");
    assert_eq!(
        accept(
            "Inspecting code",
            "Investigating missing Herdr status messages",
            &directions
        ),
        Ok("Investigating...".into())
    );
    assert_eq!(
        accept("Inspecting code", &"é".repeat(21), &directions),
        Ok(format!("{}...", "é".repeat(17)))
    );
    assert!(
        accept(
            "Inspecting code",
            "Return only that summary on one line in 20 characters",
            &directions
        )
        .is_err()
    );
    assert!(accept("Inspecting code", " \n", &directions).is_err());
    assert!(accept("Inspecting code", "Editing \x1b[31mthe parser", &directions).is_err());
    assert_eq!(
        accept(
            "Editing the parser implementation",
            "Summarize the current status of the parser implementation",
            &directions
        ),
        Ok("Summarize the...".into())
    );
}

#[test]
fn prompts_state_the_enforced_limit() {
    let stated = format!("{MAX_CHARS} characters or fewer");
    assert!(directions("user request").contains(&stated));
    assert!(directions("agent's current status").contains(&stated));
}

#[test]
fn status_excerpt_keeps_the_recent_tail() {
    assert_eq!(status_excerpt("  \n"), None);
    assert_eq!(status_excerpt(" Reading "), Some("Reading".into()));
    let long = format!("{}TAIL", "x".repeat(4_010));
    let excerpt = status_excerpt(&long).unwrap();
    assert_eq!(excerpt.chars().count(), 4_000);
    assert!(excerpt.ends_with("TAIL"));
}
