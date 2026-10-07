use std::time::Duration;

use super::{Ledger, absorb, cursor_from, sync_cursor};
use crate::config::malvin_config_file::TokenCostRates;
use crate::run_timing::RunTiming;
use crate::run_timing::report::format_timing_stdout_line_from_json;
use crate::run_timing::report_cost_line::format_cost_stdout_line_from_json;

#[test]
fn absorb_sums_new_work_without_recounting_carried_tokens() {
    let mut totals = Ledger::default();
    let mut first = RunTiming {
        tokens_in: Some(10),
        llm_wait: Duration::from_millis(100),
        tool_calls: Duration::from_millis(5),
        tool_calls_read: Duration::from_millis(5),
        ..RunTiming::default()
    };
    absorb(&mut first, &mut totals);

    let mut second = RunTiming::default();
    second.carry_token_and_cost_from(&first);
    sync_cursor(&mut second);
    second.tokens_in = Some(15);
    second.llm_wait = Duration::from_millis(40);
    second.tool_calls = Duration::from_millis(7);
    second.tool_calls_edit = Duration::from_millis(7);
    absorb(&mut second, &mut totals);

    assert_eq!(totals.tokens_in, Some(15));
    assert_eq!(totals.llm_wait, Duration::from_millis(140));
    assert_eq!(totals.tool_calls, Duration::from_millis(12));
    assert_eq!(totals.read, Duration::from_millis(5));
    assert_eq!(totals.edit, Duration::from_millis(7));
}

#[test]
fn second_absorb_on_the_same_timer_adds_only_the_increment() {
    let mut totals = Ledger::default();
    let mut timing = RunTiming {
        tokens_out: Some(3),
        llm_wait: Duration::from_millis(10),
        ..RunTiming::default()
    };
    absorb(&mut timing, &mut totals);
    timing.tokens_out = Some(8);
    timing.llm_wait = Duration::from_millis(25);
    absorb(&mut timing, &mut totals);
    assert_eq!(totals.tokens_out, Some(8));
    assert_eq!(totals.llm_wait, Duration::from_millis(25));
}

#[test]
fn absorb_sums_estimated_dollars_across_slices() {
    let mut totals = Ledger::default();
    let mut timing = RunTiming {
        token_cost_rates: TokenCostRates {
            usd_per_microtoken_in: 1_000.0,
            ..TokenCostRates::default()
        },
        tokens_in: Some(1_000_000),
        ..RunTiming::default()
    };
    absorb(&mut timing, &mut totals);
    timing.tokens_in = Some(2_000_000);
    absorb(&mut timing, &mut totals);
    let cost_in = totals.cost_in.expect("cost");
    assert!((cost_in - 2_000.0).abs() < 1e-9, "cost_in={cost_in}");
    let cost_tot = totals.cost_tot.expect("total");
    assert!((cost_tot - 2_000.0).abs() < 1e-9, "cost_tot={cost_tot}");
}

#[test]
fn footnote_lines_use_process_wall_and_summed_tokens() {
    let totals = Ledger {
        llm_wait: Duration::from_millis(1500),
        tokens_in: Some(15),
        steps: 2,
        cost_in: Some(0.5),
        cost_out: Some(0.0),
        cost_read: Some(0.0),
        cost_write: Some(0.0),
        cost_tot: Some(0.5),
        ..Ledger::default()
    };
    let json = super::footnotes_value(&totals, 2500);
    let timing = format_timing_stdout_line_from_json(&json);
    let cost = format_cost_stdout_line_from_json(&json);
    assert!(timing.contains("wall = 2.5s"), "{timing}");
    assert!(timing.contains("llm_wait = 1.5s"), "{timing}");
    assert!(cost.contains("steps = 2"), "{cost}");
    assert!(cost.contains("tokens_in = 15"), "{cost}");
    assert!(cost.contains("cost_tot = 0.5000"), "{cost}");
}

#[test]
fn missing_usage_stays_na_on_the_cost_line() {
    let json = super::footnotes_value(&Ledger::default(), 0);
    let cost = format_cost_stdout_line_from_json(&json);
    assert!(cost.contains("tokens_in = n/a"), "{cost}");
    assert!(cost.contains("cost_tot = n/a"), "{cost}");
}

#[test]
fn attach_syncs_the_cursor_to_carried_tokens() {
    let mut slot = None;
    let first = crate::run_timing::attach_new_run_timing(&mut slot, "cursor:auto");
    first.lock().expect("lock").tokens_in = Some(10);
    let second = crate::run_timing::attach_new_run_timing(&mut slot, "cursor:auto");
    let cursor_tokens = {
        let guard = second.lock().expect("lock");
        assert_eq!(guard.tokens_in, Some(10));
        assert_eq!(guard.ledger_cursor.tokens_in, Some(10));
        let tokens = cursor_from(&guard).tokens_in;
        drop(guard);
        tokens
    };
    assert_eq!(cursor_tokens, Some(10));
}

#[test]
fn armed_emit_prints_timing_and_cost_outside_dm_mode() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    crate::output::set_stdout_log_path(None);
    crate::output::set_do_dm_stdout_mode(true);
    crate::output::enable_stdout_capture();
    super::arm_process_footnotes();
    super::emit_process_footnotes_if_armed();
    let text = crate::output::take_captured_stdout();
    crate::output::enable_stdout_capture();
    super::emit_process_footnotes_if_armed();
    let second = crate::output::take_captured_stdout();
    crate::output::set_do_dm_stdout_mode(false);
    assert!(text.contains("TIMING:"), "{text}");
    assert!(text.contains("COST:"), "{text}");
    assert!(!second.contains("TIMING:"), "{second}");
}
