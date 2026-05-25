use std::sync::OnceLock;

use rand::seq::SliceRandom;

const HEARTBEAT_PHRASES_RAW: &str = include_str!("../default_prompts/heartbeats.txt");

#[must_use]
pub fn timestamp_now_string() -> String {
    let now = chrono::Local::now();
    format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    )
}

fn heartbeat_phrases() -> &'static [&'static str] {
    static PHRASES: OnceLock<Vec<&'static str>> = OnceLock::new();
    PHRASES.get_or_init(|| {
        HEARTBEAT_PHRASES_RAW
            .lines()
            .filter(|line| !line.trim().is_empty())
            .collect()
    })
    .as_slice()
}

fn random_heartbeat_phrase() -> &'static str {
    let phrases = heartbeat_phrases();
    phrases
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or("Still alive, still alive.")
}

/// Wall-clock payload for stdout heartbeats (`YYYYMMDD.HHMMSS {phrase}`).
#[must_use]
pub fn heartbeat_payload_now() -> String {
    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d.%H%M%S");
    format!("{ts} {}", random_heartbeat_phrase())
}

#[must_use]
pub fn heartbeat_payload_has_wall_clock_prefix(payload: &str) -> bool {
    let Some(ts) = payload.get(..15) else {
        return false;
    };
    if ts.len() != 15 || ts.as_bytes().get(8) != Some(&b'.') {
        return false;
    }
    ts.chars()
        .all(|c| c.is_ascii_digit() || c == '.')
        && payload.get(15..16) == Some(" ")
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_timestamp_now_string() {
        let _ = super::timestamp_now_string;
        let _ = super::heartbeat_payload_now;
    }

    #[test]
    fn heartbeat_payload_now_starts_with_wall_clock_timestamp() {
        let payload = super::heartbeat_payload_now();
        assert!(super::heartbeat_payload_has_wall_clock_prefix(&payload));
    }

    #[test]
    fn heartbeat_phrases_ignore_blank_lines() {
        let phrases = super::heartbeat_phrases();
        assert!(!phrases.is_empty());
        assert!(phrases.iter().all(|p| !p.trim().is_empty()));
    }

    #[test]
    fn heartbeat_payload_has_wall_clock_prefix_accepts_new_shape() {
        assert!(super::heartbeat_payload_has_wall_clock_prefix(
            "20260524.000000 Still alive."
        ));
        assert!(!super::heartbeat_payload_has_wall_clock_prefix("HB: old"));
    }

    #[test]
    fn kiss_cov_random_heartbeat_phrase() {
        let _ = super::random_heartbeat_phrase();
    }
}
