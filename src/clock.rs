use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub trait Clock {
    fn unix_now_secs(&self) -> u64;
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn unix_now_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[must_use]
pub fn unix_now_secs() -> u64 {
    SystemClock.unix_now_secs()
}

#[must_use]
pub fn fetched_at_is_fresh(clock: &impl Clock, fetched_at_secs: u64, ttl: Duration) -> bool {
    clock.unix_now_secs().saturating_sub(fetched_at_secs) < ttl.as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FixedClock(u64);

    impl Clock for FixedClock {
        fn unix_now_secs(&self) -> u64 {
            self.0
        }
    }
    #[test]
    fn system_clock_is_after_2023_and_freshness_boundary_is_exclusive_at_ttl() {
        {
            assert!(unix_now_secs() > 1_700_000_000);
        }
        {
            let ttl = Duration::from_secs(100);
            let clock = FixedClock(1_000);
            assert!(fetched_at_is_fresh(&clock, 901, ttl));
            assert!(!fetched_at_is_fresh(&clock, 900, ttl));
            assert!(fetched_at_is_fresh(&clock, 2_000, ttl));
        }
    }
}
