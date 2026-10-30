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

#[derive(Clone, Copy)]
pub(super) struct StatusSample {
    pub now: Instant,
    pub due: Instant,
    pub seen_prompts: u64,
    pub prompt_count: u64,
}

pub(super) struct StatusWake {
    pub generate: bool,
    pub timer_due: bool,
    pub seen_prompts: u64,
}

pub(super) fn status_wake(sample: StatusSample) -> StatusWake {
    let prompt_due = sample.prompt_count > sample.seen_prompts;
    let timer_due = sample.now >= sample.due;
    let seen_prompts = if prompt_due {
        sample.seen_prompts.saturating_add(1)
    } else {
        sample.seen_prompts
    };
    StatusWake {
        generate: prompt_due || timer_due,
        timer_due,
        seen_prompts,
    }
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
    fn status_wake_fires_for_a_new_prompt_or_a_due_timer() {
        let now = Instant::now();
        let due = now + Duration::from_secs(300);
        let waiting = status_wake(StatusSample {
            now,
            due,
            seen_prompts: 12,
            prompt_count: 12,
        });
        assert!(!waiting.generate);
        assert!(!waiting.timer_due);
        assert_eq!(waiting.seen_prompts, 12);

        let first = status_wake(StatusSample {
            now,
            due,
            seen_prompts: 12,
            prompt_count: 14,
        });
        assert!(first.generate);
        assert!(!first.timer_due);
        assert_eq!(first.seen_prompts, 13);
        let second = status_wake(StatusSample {
            now,
            due,
            seen_prompts: first.seen_prompts,
            prompt_count: 14,
        });
        assert_eq!(second.seen_prompts, 14);
        let done = status_wake(StatusSample {
            now,
            due,
            seen_prompts: second.seen_prompts,
            prompt_count: 14,
        });
        assert!(!done.generate);

        let timed = status_wake(StatusSample {
            now: due,
            due,
            seen_prompts: 14,
            prompt_count: 14,
        });
        assert!(timed.generate);
        assert!(timed.timer_due);
        assert_eq!(timed.seen_prompts, 14);

        let shrunk = status_wake(StatusSample {
            now,
            due,
            seen_prompts: 14,
            prompt_count: 10,
        });
        assert!(!shrunk.generate);
        assert_eq!(shrunk.seen_prompts, 14);
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
