use crate::output::stdout_log_pair::{
    acp_tee_log_line, tagged_display_and_log_line_for_color, tagged_log_line, AcpTeeDirection,
    AcpTeeLineFmt,
};
use crate::output::{
    format_who_tag_delim, format_who_tag_prefix, ERROR_WHO, WARNING_WHO, WHO_B, WHO_H, WHO_M, WHO_O,
    WHO_T, WHO_U,
};

fn log_line_uses_delim_without_trailing_space(log: &str, who: &str, payload: &str) -> bool {
    let delim = format_who_tag_delim(who);
    log.ends_with(&format!("{delim}{payload}"))
}

const FUZZ_WHO_TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T, WHO_U, WHO_H];

#[test]
fn tagged_log_line_omits_space_after_who_pipe_for_all_tags() {
    const TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T, WHO_U, WHO_H, ERROR_WHO, WARNING_WHO];
    let ts = "20260524.000000.000";
    for who in TAGS {
        let payload = "probe payload";
        let log = tagged_log_line(ts, who, payload);
        assert!(
            log_line_uses_delim_without_trailing_space(&log, who, payload),
            "who={who}: {log:?}"
        );
    }
}

#[test]
fn display_log_metamorphic_pipe_space_only_for_thought_and_tool() {
    const TAGS: &[&str] = &[WHO_O, WHO_M, WHO_B, WHO_T];
    let ts = "20260524.000000.000";
    for who in TAGS {
        for payload in ["Run cargo test", "internal reasoning", ""] {
            let (display, log) =
                tagged_display_and_log_line_for_color(who, payload, Some(ts), false);
            assert_eq!(log, tagged_log_line(ts, who, payload));
            assert_eq!(display, format!("{}{payload}", format_who_tag_prefix(who)));
            assert!(log_line_uses_delim_without_trailing_space(&log, who, payload));
        }
    }
}

#[test]
fn log_rejects_decorative_space_after_who_pipe() {
    let ts = "20260524.000000.000";
    let who = WHO_O;
    let payload = "Logs: /tmp/run";
    let log = tagged_log_line(ts, who, payload);
    let delim = format_who_tag_delim(who);
    let bad = format!("{ts} {delim} {payload}");
    assert_ne!(log, bad, "log must not insert decorative space after who pipe");
    assert!(log_line_uses_delim_without_trailing_space(&log, who, payload));
}

#[test]
fn acp_tee_log_line_omits_space_after_who_pipe() {
    let payload = "Run echo hi · 1ms · ✓";
    let ctx = AcpTeeLineFmt {
        ts: "20260524.000000.000",
        direction: AcpTeeDirection::FromAgent,
        who: WHO_T,
        line: payload,
        dim_payload: true,
    };
    let log = acp_tee_log_line(&ctx);
    assert!(log_line_uses_delim_without_trailing_space(&log, WHO_T, payload), "{log:?}");
}

#[test]
fn tagged_log_line_no_pipe_space_fuzz() {
    use rand::{Rng, SeedableRng};

    let seed = std::env::var("LOG_PIPE_FUZZ_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(rand::random);
    eprintln!("tagged_log_line_no_pipe_space_fuzz seed: {seed}");
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let ts = "20260524.000000.000";
    for _ in 0..200 {
        let who = FUZZ_WHO_TAGS[rng.gen_range(0..FUZZ_WHO_TAGS.len())];
        let lead_spaces: String = (0..rng.gen_range(0..3)).map(|_| ' ').collect();
        let payload = format!("{lead_spaces}payload{}", rng.gen_range(0..u32::MAX));
        let log = tagged_log_line(ts, who, &payload);
        assert!(
            log_line_uses_delim_without_trailing_space(&log, who, &payload),
            "seed={seed} log={log:?}"
        );
    }
}
