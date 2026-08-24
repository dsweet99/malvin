use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(crate) fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn cache_fetched_at_is_fresh(fetched_at_secs: u64, ttl: Duration) -> bool {
    unix_now_secs().saturating_sub(fetched_at_secs) < ttl.as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_now_secs_is_positive() {
        assert!(unix_now_secs() > 0);
    }

    #[test]
    fn cache_freshness_respects_ttl() {
        assert!(cache_fetched_at_is_fresh(unix_now_secs(), Duration::from_hours(24)));
        let stale = unix_now_secs().saturating_sub(Duration::from_hours(25).as_secs());
        assert!(!cache_fetched_at_is_fresh(stale, Duration::from_hours(24)));
    }
}
