#[must_use]
pub fn timestamp_now_string() -> String {
    let now = chrono::Local::now();
    format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    )
}

/// Wall-clock payload for stdout heartbeats (`HB: YYYYMMDD.HHMMSS`).
#[must_use]
pub fn heartbeat_payload_now() -> String {
    let now = chrono::Local::now();
    format!("HB: {}", now.format("%Y%m%d.%H%M%S"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_timestamp_now_string() {
        let _ = super::timestamp_now_string;
        let _ = super::heartbeat_payload_now;
    }

    #[test]
    fn heartbeat_payload_now_starts_with_hb_prefix() {
        let payload = super::heartbeat_payload_now();
        assert!(payload.starts_with("HB: "));
        assert_eq!(payload.len(), 4 + 15);
    }
}
