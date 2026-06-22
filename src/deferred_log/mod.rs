mod active;
mod config;
mod emit;
mod enrich;
mod sink;
mod sink_build;
mod tool_enrich;
mod types;

#[cfg(test)]
#[path = "tests_active.rs"]
mod tests_active;

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
#[path = "tests_plan.rs"]
mod tests_plan;

#[path = "tests_cursor_store.rs"]
mod tests_cursor_store;

pub(crate) use active::{
    flush_pending_into as flush_pending_into_active_sink, register as register_active_sink,
    unregister as unregister_active_sink, SharedDeferSink,
};
pub(crate) use sink_build::{
    build_acp_tee_entry, build_display_log_entry, build_raw_line_entry, build_tool_entry,
};
pub(crate) use sink::DeferredLogSink;
pub(crate) use tool_enrich::tool_drain_enrich_fields;
pub(crate) use types::{AcpTeeBuild, DeferredEntry, TeeSinkMeta, ToolSummaryBuild};

pub(crate) fn log_with_heartbeat(sink: &mut DeferredLogSink, entry: DeferredEntry) {
    if !active::defer_already_has_heartbeat(sink) {
        if let Some((display, log)) = crate::output::heartbeat_rendered_if_due(
            std::time::Instant::now(),
            true,
        ) {
            crate::output::publish_heartbeat_live_terminal(&display);
            sink.push_entry(build_display_log_entry(display, log));
        }
    }
    sink.push_entry(entry);
}

pub(crate) fn install_stdout_hooks() {
    static INSTALLED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INSTALLED.get_or_init(|| {
        crate::output::register_defer_stdout_hooks(defer_tagged_stdout_hook, defer_heartbeat_hook);
    });
}

fn defer_tagged_stdout_hook(display: &str, log: &str) -> bool {
    active::try_log(build_display_log_entry(
        display.to_string(),
        log.to_string(),
    ))
}

fn defer_heartbeat_hook(display: &str, log: &str) -> bool {
    let show_live = !active::heartbeat_live_pending();
    if !active::try_push(build_display_log_entry(
        display.to_string(),
        log.to_string(),
    )) {
        return false;
    }
    if show_live && !active::defer_sink_mutex_held() {
        crate::output::publish_heartbeat_live_terminal(display);
    }
    true
}

#[cfg(test)]
#[path = "hook_test.rs"]
mod hook_test;

#[cfg(test)]
#[path = "deferred_log_test.rs"]
mod deferred_log_test;
