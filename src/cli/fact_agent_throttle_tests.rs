use super::{FACT_AGENT_INTERVAL_SECS as HOUR, is_due, try_claim};

#[test]
fn claims_once_per_hour_and_treats_corrupt_record_as_due() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.json");
    assert!(try_claim(&path, 1000));
    assert!(!try_claim(&path, 1000 + HOUR - 1));
    assert!(try_claim(&path, 1000 + HOUR));
    std::fs::write(&path, "not json").unwrap();
    assert!(is_due(&path, 5));
}
