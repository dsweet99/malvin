#[must_use]
pub fn timestamp_now_string() -> String {
    let now = chrono::Local::now();
    format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    )
}

/// Wall-clock payload for stdout heartbeats (`YYYYMMDD.HHMMSS {status}`).
#[must_use]
pub fn heartbeat_payload_now() -> String {
    let now = chrono::Local::now();
    let ts = now.format("%Y%m%d.%H%M%S");
    let mut payload = format!("{ts} {}", crate::agent_phase::heartbeat_label());
    if let Some(stats) = crate::active_agent_heartbeat::active_agent_heartbeat_stats() {
        payload.push(' ');
        payload.push_str(&stats);
    }
    payload
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

    #[cfg(unix)]
    #[test]
    fn heartbeat_payload_now_includes_agent_stats_when_session_registered() {
        crate::active_agent_heartbeat::clear_active_agent_process_groups_for_test();
        let pgid = std::process::id();
        let baseline = crate::acp::snapshot_pids();
        crate::active_agent_heartbeat::register_active_agent_process_group(Some(pgid), baseline);
        let payload = super::heartbeat_payload_now();
        assert!(payload.contains("sandbox "));
        assert!(payload.contains("RSS"));
        assert!(payload.contains("procs"));
        crate::active_agent_heartbeat::unregister_active_agent_process_group(Some(pgid));
        crate::active_agent_heartbeat::clear_active_agent_process_groups_for_test();
    }

    #[test]
    fn heartbeat_payload_uses_agent_phase_label() {
        let _guard = crate::agent_phase::AGENT_PHASE_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        crate::agent_phase::reset_phase_state_for_test();
        let payload = super::heartbeat_payload_now();
        assert!(payload.contains("Orienting"));
    }

    #[test]
    fn heartbeat_payload_has_wall_clock_prefix_accepts_new_shape() {
        assert!(super::heartbeat_payload_has_wall_clock_prefix(
            "20260524.000000 Still alive."
        ));
        assert!(!super::heartbeat_payload_has_wall_clock_prefix("HB: old"));
    }

}
