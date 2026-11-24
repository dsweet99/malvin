use super::{FACT_AGENT_INTERVAL_SECS as HOUR, is_due, try_claim};

#[test]
fn claims_once_per_hour_even_when_racing_and_treats_corrupt_record_as_due() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("r.json");
    assert!(try_claim(&path, 1000));
    assert!(!try_claim(&path, 1000 + HOUR - 1));
    assert!(try_claim(&path, 1000 + HOUR));
    std::fs::write(&path, "not json").unwrap();
    assert!(is_due(&path, 5));
    for _ in 0..50 {
        let dir = tempfile::tempdir().unwrap();
        let path = std::sync::Arc::new(dir.path().join("r.json"));
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let (p, b) = (path.clone(), barrier.clone());
                std::thread::spawn(move || {
                    b.wait();
                    try_claim(&p, 5000)
                })
            })
            .collect();
        let wins = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .filter(|w| *w)
            .count();
        assert_eq!(wins, 1, "claimants that won the same hour");
        assert!(!is_due(&path, 5000 + HOUR - 1));
    }
}
