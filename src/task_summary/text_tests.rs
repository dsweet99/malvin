use super::*;

#[test]
fn short_requests_are_clipped_at_a_word_and_marked() {
    assert_eq!(
        local_task("Investigate extraordinarily complicated parsing failures").as_deref(),
        Some("Investigate extraordinarily...")
    );
    assert_eq!(
        local_task("One two three four five six seven eight nine").as_deref(),
        Some("One two three four five six...")
    );
    assert_eq!(
        local_task("Réparer immédiatement les caractères Unicode du résumé").as_deref(),
        Some("Réparer immédiatement les...")
    );
    assert_eq!(
        local_task("Fix the parser tonight.").as_deref(),
        Some("Fix the parser tonight.")
    );
}

#[test]
fn summary_is_at_most_thirty_characters() {
    assert_eq!(
        normalize("\"Fix the parser\""),
        Some("Fix the parser".into())
    );
    assert_eq!(
        normalize("Add\ncheap request summaries"),
        Some("Add cheap request summaries".into())
    );
    assert_eq!(
        normalize("123456789012345678901234567890"),
        Some("123456789012345678901234567890".into())
    );
    assert_eq!(normalize("1234567890123456789012345678901"), None);
    assert_eq!(normalize("one two \x1b[31mthree"), None);
    assert_eq!(normalize(" \n"), None);
}

#[test]
fn summary_must_describe_the_source_rather_than_the_directions() {
    let directions = directions("user request");
    assert!(accept("Thanks", "Return only that summary", &directions).is_err());
    assert!(accept("Thanks", "Use 30 characters now", &directions).is_err());
    assert_eq!(
        accept("Thanks", "\"User expresses thanks\"", &directions).unwrap(),
        "User expresses thanks"
    );
    assert_eq!(
        accept(
            "Show the 30-character limit",
            "Show the 30-character limit",
            &directions
        )
        .unwrap(),
        "Show the 30-character limit"
    );
}

#[test]
fn stored_direction_text_is_not_publishable() {
    assert_eq!(publishable("Return only that summary"), None);
    assert_eq!(
        publishable("Add cheap request summaries"),
        Some("Add cheap request summaries".into())
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
        Ok("Investigating missing Herdr...".into())
    );
    assert_eq!(
        accept("Inspecting code", &"é".repeat(31), &directions),
        Ok(format!("{}...", "é".repeat(27)))
    );
    assert!(
        accept(
            "Inspecting code",
            "Return only that summary on one line in 30 characters",
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
        Ok("Summarize the current...".into())
    );
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
