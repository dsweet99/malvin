use super::{run_models_with_gates, with_bridge};

fn run_usage_case(models: &str, expected: (u64, u64, f64, u64)) {
    with_bridge(|work| {
        let config = std::path::PathBuf::from(std::env::var_os("HOME").unwrap())
            .join(".malvinconf/config.toml");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::write(
            config,
            "[agent.cursor.metered-poor]\nusd_per_microtoken_in = 1000\n\
             [agent.cursor.metered-error]\nusd_per_microtoken_in = 1000\n\
             [agent.cursor.late-poor]\nusd_per_microtoken_in = 1000\n\
             [agent.cursor.late-poor-again]\nusd_per_microtoken_in = 2000\n\
             [agent.cursor.summary-poor]\nusd_per_microtoken_in = 1000\n\
             [agent.cursor.good]\nusd_per_microtoken_in = 3000\n",
        )
        .unwrap();
        let artifacts = run_models_with_gates(work, models, false).unwrap();
        let report: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(artifacts.run_dir.join("run_timing.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["tokens"]["tokens_in"], expected.0, "{report}");
        assert_eq!(report["tokens"]["tokens_out"], expected.1, "{report}");
        assert!(
            (report["cost"]["cost_tot"].as_f64().unwrap() - expected.2).abs() < 1e-9,
            "{report}"
        );
        assert_eq!(report["tokens"]["usage_tx_count"], expected.3, "{report}");
        // A second task in the same process must start with fresh totals.
        let fresh = run_models_with_gates(work, "cursor:good", false).unwrap();
        let fresh: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(fresh.run_dir.join("run_timing.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(fresh["tokens"]["tokens_in"], 30);
        assert!((fresh["cost"]["cost_tot"].as_f64().unwrap() - 0.09).abs() < 1e-9);
    });
}

#[test]
fn late_fallback_retains_usage_and_each_models_rates() {
    run_usage_case("cursor:late-poor,cursor:good", (130, 4, 0.19, 4));
}

#[test]
fn summary_fallback_retains_usage_and_each_models_rates() {
    run_usage_case("cursor:summary-poor,cursor:good", (50, 5, 0.11, 5));
}

#[test]
fn multiple_fallbacks_retain_all_attempts_without_double_counting() {
    run_usage_case(
        "cursor:late-poor,cursor:late-poor-again,cursor:good",
        (230, 5, 0.39, 5),
    );
}

#[test]
fn fatal_quota_fallback_preserves_streamed_usage() {
    run_usage_case("cursor:metered-poor,cursor:good", (100, 10, 0.16, 4));
}

#[test]
fn error_completion_quota_fallback_preserves_streamed_usage() {
    run_usage_case("cursor:metered-error,cursor:good", (100, 10, 0.16, 4));
}
