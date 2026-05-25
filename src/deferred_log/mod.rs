mod active;
mod config;
mod emit;
mod enrich;
mod sink;
mod tool_enrich;
mod types;

#[cfg(test)]
mod kiss_coverage;
#[cfg(test)]
#[path = "tests_active.rs"]
mod tests_active;

#[cfg(test)]
mod test_fixtures;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests_plan.rs"]
mod tests_plan;

#[cfg(test)]
#[path = "tests_cursor_store.rs"]
mod tests_cursor_store;

pub(crate) use active::{
    flush_pending_into as flush_pending_into_active_sink,
    is_registered as active_defer_sink_registered, register as register_active_sink,
    unregister as unregister_active_sink, SharedDeferSink,
};
pub(crate) use sink::{
    build_acp_tee_entry, build_heartbeat_entry, build_raw_line_entry, build_tagged_stdout_entry,
    build_tool_entry, DeferredLogSink,
};
pub(crate) use tool_enrich::tool_drain_enrich_fields;
pub(crate) use types::{AcpTeeBuild, DeferredEntry, TeeSinkMeta, ToolSummaryBuild};

pub(crate) fn log_with_heartbeat(sink: &mut DeferredLogSink, entry: DeferredEntry) {
    if let Some(log_line) = crate::output::heartbeat_log_line_for_defer_sink(
        std::time::Instant::now(),
        true,
    ) {
        sink.push_entry(build_heartbeat_entry(log_line));
    }
    sink.push_entry(entry);
}

pub(crate) fn install_stdout_hooks() {
    static INSTALLED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    INSTALLED.get_or_init(|| {
        crate::output::register_defer_stdout_hooks(defer_tagged_stdout_hook, defer_push_hook);
    });
}

fn defer_tagged_stdout_hook(display: &str, log: &str) -> bool {
    active::try_log(build_tagged_stdout_entry(
        display.to_string(),
        log.to_string(),
    ))
}

fn defer_push_hook(log_line: String) -> bool {
    active::try_push(build_heartbeat_entry(log_line))
}

#[cfg(test)]
mod hook_tests {
    use super::{
        defer_push_hook, defer_tagged_stdout_hook, register_active_sink, unregister_active_sink,
        DeferredLogSink, SharedDeferSink,
    };
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn defer_hooks_invoke_active_paths() {
        let shared: SharedDeferSink = Arc::new(std::sync::Mutex::new(DeferredLogSink::new(
            "hook".to_string(),
            PathBuf::new(),
            super::config::DeferredLogConfig::from_env(),
        )));
        register_active_sink(Arc::clone(&shared));
        assert!(defer_tagged_stdout_hook("d", "l"));
        assert!(defer_push_hook("hb".to_string()));
        unregister_active_sink();
        assert!(!defer_tagged_stdout_hook("d", "l"));
        assert!(!defer_push_hook("hb".to_string()));
    }
}
