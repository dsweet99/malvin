use std::time::{Duration, Instant};

const FIRST_STATUS: Duration = Duration::from_mins(5);
const STATUS_PERIOD: Duration = Duration::from_mins(10);

pub(super) fn status_delays() -> (Duration, Duration) {
    test_delays().unwrap_or((FIRST_STATUS, STATUS_PERIOD))
}

fn test_delays() -> Option<(Duration, Duration)> {
    std::env::var_os("MALVIN_TEST_NO_REAL_AGENT")?;
    let spec = std::env::var("MALVIN_TEST_STATUS_SCHEDULE_MS").ok()?;
    parse_schedule_ms(&spec)
}

pub(super) fn parse_schedule_ms(spec: &str) -> Option<(Duration, Duration)> {
    let (first, period) = spec.split_once(',')?;
    let first = first.trim().parse::<u64>().ok()?;
    let period = period.trim().parse::<u64>().ok()?;
    if first == 0 || period == 0 {
        return None;
    }
    Some((Duration::from_millis(first), Duration::from_millis(period)))
}

pub(super) fn next_due(due: Instant, period: Duration, now: Instant) -> Instant {
    let mut next = due + period;
    while next <= now {
        next += period;
    }
    next
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_schedule_starts_at_five_minutes_and_repeats_every_ten() {
        assert_eq!(FIRST_STATUS, Duration::from_secs(300));
        assert_eq!(STATUS_PERIOD, Duration::from_secs(600));
        assert_eq!(
            parse_schedule_ms("80,120"),
            Some((Duration::from_millis(80), Duration::from_millis(120)))
        );
        assert_eq!(parse_schedule_ms("0,120"), None);
        assert_eq!(parse_schedule_ms("80"), None);
    }

    #[test]
    fn next_due_skips_deadlines_already_passed() {
        let start = Instant::now();
        let due = start + Duration::from_secs(300);
        let period = Duration::from_secs(600);
        assert_eq!(next_due(due, period, start), due + period);
        let late = due + period + Duration::from_secs(1);
        assert_eq!(next_due(due, period, late), due + period * 2);
    }
}
