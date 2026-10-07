use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::RunTiming;

static START: OnceLock<Instant> = OnceLock::new();
static ARMED: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug, Default)]
pub(crate) struct Ledger {
    pub(crate) llm_wait: Duration,
    pub(crate) tool_calls: Duration,
    pub(crate) read: Duration,
    pub(crate) search: Duration,
    pub(crate) edit: Duration,
    pub(crate) execute: Duration,
    pub(crate) other: Duration,
    pub(crate) steps: u64,
    pub(crate) tokens_in: Option<u64>,
    pub(crate) tokens_out: Option<u64>,
    pub(crate) cache_read: Option<u64>,
    pub(crate) cache_write: Option<u64>,
    pub(crate) reasoning: Option<u64>,
    pub(crate) cost_in: Option<f64>,
    pub(crate) cost_out: Option<f64>,
    pub(crate) cost_read: Option<f64>,
    pub(crate) cost_write: Option<f64>,
    pub(crate) cost_tot: Option<f64>,
}

pub fn note_process_start() {
    let _ = START.get_or_init(Instant::now);
}

pub fn arm_process_footnotes() {
    note_process_start();
    ARMED.store(true, Ordering::Relaxed);
}

pub fn emit_process_footnotes_if_armed() {
    if !ARMED.swap(false, Ordering::Relaxed) {
        return;
    }
    note_process_start();
    let totals = totals_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let wall = START.get().map_or(0, |start| ms(start.elapsed()));
    let restore_dm = crate::output::do_dm_stdout_mode();
    crate::output::set_do_dm_stdout_mode(false);
    super::report::print_timing_and_cost_summary(&footnotes_value(&totals, wall));
    if restore_dm {
        crate::output::set_do_dm_stdout_mode(true);
    }
}

fn totals_slot() -> &'static Mutex<Ledger> {
    static TOTALS: OnceLock<Mutex<Ledger>> = OnceLock::new();
    TOTALS.get_or_init(|| Mutex::new(Ledger::default()))
}

pub(crate) fn sync_cursor(timing: &mut RunTiming) {
    timing.ledger_cursor = cursor_from(timing);
}

pub(crate) fn contribute(timing: &mut RunTiming) {
    let mut totals = totals_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    absorb(timing, &mut totals);
}

pub(crate) fn absorb(timing: &mut RunTiming, totals: &mut Ledger) {
    let now = cursor_from(timing);
    fold_durations(totals, &now, &timing.ledger_cursor);
    fold_counts(totals, &now, &timing.ledger_cursor);
    timing.ledger_cursor = now;
}

fn cursor_from(timing: &RunTiming) -> Ledger {
    let mut ledger = Ledger {
        llm_wait: timing.llm_wait,
        tool_calls: timing.tool_calls,
        read: timing.tool_calls_read,
        search: timing.tool_calls_search,
        edit: timing.tool_calls_edit,
        execute: timing.tool_calls_execute,
        other: timing.tool_calls_other,
        steps: timing.steps,
        tokens_in: timing.tokens_in,
        tokens_out: timing.tokens_out,
        cache_read: timing.cache_read,
        cache_write: timing.cache_write,
        reasoning: timing.reasoning_tokens,
        ..Ledger::default()
    };
    apply_cost(&mut ledger, timing);
    ledger
}

fn apply_cost(ledger: &mut Ledger, timing: &RunTiming) {
    let Some(cost) = super::cost::cost_stats(timing) else {
        return;
    };
    ledger.cost_in = json_f64(&cost, "cost_in");
    ledger.cost_out = json_f64(&cost, "cost_out");
    ledger.cost_read = json_f64(&cost, "cost_read");
    ledger.cost_write = json_f64(&cost, "cost_write");
    ledger.cost_tot = json_f64(&cost, "cost_tot");
}

fn json_f64(value: &serde_json::Value, key: &str) -> Option<f64> {
    value.get(key).and_then(serde_json::Value::as_f64)
}

const fn fold_durations(totals: &mut Ledger, now: &Ledger, was: &Ledger) {
    add_dur(&mut totals.llm_wait, now.llm_wait, was.llm_wait);
    add_dur(&mut totals.tool_calls, now.tool_calls, was.tool_calls);
    add_dur(&mut totals.read, now.read, was.read);
    add_dur(&mut totals.search, now.search, was.search);
    add_dur(&mut totals.edit, now.edit, was.edit);
    add_dur(&mut totals.execute, now.execute, was.execute);
    add_dur(&mut totals.other, now.other, was.other);
}

const fn add_dur(slot: &mut Duration, now: Duration, was: Duration) {
    *slot = slot.saturating_add(now.saturating_sub(was));
}

fn fold_counts(totals: &mut Ledger, now: &Ledger, was: &Ledger) {
    totals.steps = totals
        .steps
        .saturating_add(now.steps.saturating_sub(was.steps));
    add_u64(&mut totals.tokens_in, now.tokens_in, was.tokens_in);
    add_u64(&mut totals.tokens_out, now.tokens_out, was.tokens_out);
    add_u64(&mut totals.cache_read, now.cache_read, was.cache_read);
    add_u64(&mut totals.cache_write, now.cache_write, was.cache_write);
    add_u64(&mut totals.reasoning, now.reasoning, was.reasoning);
    add_f64_opt(&mut totals.cost_in, now.cost_in, was.cost_in);
    add_f64_opt(&mut totals.cost_out, now.cost_out, was.cost_out);
    add_f64_opt(&mut totals.cost_read, now.cost_read, was.cost_read);
    add_f64_opt(&mut totals.cost_write, now.cost_write, was.cost_write);
    add_f64_opt(&mut totals.cost_tot, now.cost_tot, was.cost_tot);
}

fn add_u64(slot: &mut Option<u64>, now: Option<u64>, was: Option<u64>) {
    let Some(delta) = delta_u64(now, was) else {
        return;
    };
    *slot = Some(slot.unwrap_or(0).saturating_add(delta));
}

const fn delta_u64(now: Option<u64>, was: Option<u64>) -> Option<u64> {
    match (now, was) {
        (Some(current), Some(prior)) => Some(current.saturating_sub(prior)),
        (Some(current), None) => Some(current),
        _ => None,
    }
}

fn add_f64_opt(slot: &mut Option<f64>, now: Option<f64>, was: Option<f64>) {
    let Some(delta) = delta_f64(now, was) else {
        return;
    };
    *slot = Some(slot.unwrap_or(0.0) + delta);
}

fn delta_f64(now: Option<f64>, was: Option<f64>) -> Option<f64> {
    match (now, was) {
        (Some(current), Some(prior)) => Some(current - prior),
        (Some(current), None) => Some(current),
        _ => None,
    }
}

fn footnotes_value(totals: &Ledger, wall_ms: u64) -> serde_json::Value {
    let mut value = serde_json::json!({
        "wall_clock_ms": wall_ms,
        "llm_wait_ms": ms(totals.llm_wait),
        "tool_calls_ms": ms(totals.tool_calls),
        "tool_calls_by_type_ms": {
            "read": ms(totals.read),
            "search": ms(totals.search),
            "edit": ms(totals.edit),
            "execute": ms(totals.execute),
            "other": ms(totals.other),
        },
        "tokens": {
            "steps": totals.steps,
            "tokens_in": opt_u64(totals.tokens_in),
            "tokens_out": opt_u64(totals.tokens_out),
            "cache_read": opt_u64(totals.cache_read),
            "cache_write": opt_u64(totals.cache_write),
            "reasoning": opt_u64(totals.reasoning),
        }
    });
    if let Some(cost) = cost_value(totals)
        && let Some(map) = value.as_object_mut()
    {
        map.insert("cost".into(), cost);
    }
    value
}

fn cost_value(totals: &Ledger) -> Option<serde_json::Value> {
    if totals.cost_in.is_none()
        && totals.cost_out.is_none()
        && totals.cost_read.is_none()
        && totals.cost_write.is_none()
        && totals.cost_tot.is_none()
    {
        return None;
    }
    Some(serde_json::json!({
        "cost_in": totals.cost_in.unwrap_or(0.0),
        "cost_out": totals.cost_out.unwrap_or(0.0),
        "cost_read": totals.cost_read.unwrap_or(0.0),
        "cost_write": totals.cost_write.unwrap_or(0.0),
        "cost_tot": totals.cost_tot.unwrap_or(0.0),
    }))
}

fn opt_u64(value: Option<u64>) -> serde_json::Value {
    value.map_or(serde_json::Value::Null, serde_json::Value::from)
}

fn ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
#[path = "process_ledger_tests.rs"]
mod process_ledger_tests;
