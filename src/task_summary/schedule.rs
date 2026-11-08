use std::time::Duration;

const RAMP_DELTAS_SECS: [u64; 6] = [5, 10, 20, 40, 80, 160];
const STEADY_SECS: u64 = 300;

pub(super) fn status_offset(index: usize) -> Duration {
    if let Some((first, period)) = test_delays() {
        return first + period * u32::try_from(index).unwrap_or(u32::MAX);
    }
    let ramp_len = RAMP_DELTAS_SECS.len();
    let ramp_used = (index + 1).min(ramp_len);
    let ramp: u64 = RAMP_DELTAS_SECS[..ramp_used].iter().sum();
    let steady_runs = u64::try_from((index + 1).saturating_sub(ramp_len)).unwrap_or(u64::MAX);
    Duration::from_secs(ramp.saturating_add(STEADY_SECS.saturating_mul(steady_runs)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_offsets_are_cumulative_sums_of_the_deltas() {
        let secs: Vec<u64> = (0..10).map(|i| status_offset(i).as_secs()).collect();
        assert_eq!(secs, [5, 15, 35, 75, 155, 315, 615, 915, 1215, 1515]);
    }

    #[test]
    fn parse_schedule_ms_validates_input() {
        assert_eq!(
            parse_schedule_ms("80,120"),
            Some((Duration::from_millis(80), Duration::from_millis(120)))
        );
        assert_eq!(parse_schedule_ms("0,120"), None);
        assert_eq!(parse_schedule_ms("80,0"), None);
        assert_eq!(parse_schedule_ms("80"), None);
    }
}
