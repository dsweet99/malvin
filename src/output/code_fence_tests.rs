use super::CodeFence;

fn open_after(lines: &[&str]) -> bool {
    let mut fence = CodeFence::default();
    for line in lines {
        fence.observe(line);
    }
    fence.is_open()
}

#[test]
fn backtick_and_tilde_fences_open_and_close() {
    assert!(open_after(&["```rust"]));
    assert!(!open_after(&["```rust", "code", "```"]));
    assert!(!open_after(&["  ~~~", "x", "~~~"]));
    assert!(!open_after(&["``", "not a fence"]));
}

#[test]
fn a_fence_closes_only_on_the_same_character_with_at_least_its_length() {
    assert!(open_after(&["~~~", "```"]));
    assert!(!open_after(&["~~~", "```", "~~~"]));
    assert!(open_after(&["````", "```"]));
    assert!(!open_after(&["````", "```", "`````"]));
    assert!(open_after(&["```", "``` not a close"]));
}

#[test]
fn observe_reports_only_lines_that_change_state() {
    let mut fence = CodeFence::default();
    assert!(!fence.observe("text"));
    assert!(!fence.observe("```a`b"));
    assert!(fence.observe("```"));
    assert!(!fence.observe("~~~"));
    assert!(fence.observe("````"));
    assert!(!fence.is_open());
}
