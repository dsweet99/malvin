use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::session_types::PromptTraceWriter;
use crate::deferred_log::{
    flush_pending_into_active_sink, register_active_sink, unregister_active_sink, DeferredLogSink,
    SharedDeferSink,
};

#[allow(clippy::struct_excessive_bools)]
pub(crate) struct LivePromptTraceArgs {
    pub file: tokio::fs::File,
    pub who: String,
    pub plain_lines: bool,
    pub stdout_replacement: Option<&'static str>,
    pub trace_raw_output: bool,
    pub show_thoughts_on_stdout: bool,
    pub emit_stdout_markdown: bool,
    pub work_dir: PathBuf,
    pub run_timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    pub session_id: String,
}

impl PromptTraceWriter {
    pub(crate) fn flush_deferred(&mut self) {
        if let Some(sink) = self.deferred_sink.take() {
            let mut guard = sink
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            flush_pending_into_active_sink(&mut guard);
            unregister_active_sink();
            guard.force_flush();
        }
    }

    fn register_deferred_sink(sink: DeferredLogSink) -> SharedDeferSink {
        let shared = Arc::new(Mutex::new(sink));
        register_active_sink(Arc::clone(&shared));
        shared
    }

    pub(crate) fn for_live_prompt(args: LivePromptTraceArgs) -> Self {
        let deferred_sink = DeferredLogSink::for_prompt(
            args.session_id.clone(),
            args.work_dir.clone(),
        )
        .map(Self::register_deferred_sink);
        Self {
            file: args.file,
            who: args.who,
            plain_lines: args.plain_lines,
            stdout_replacement: args.stdout_replacement,
            placeholder_emitted: false,
            raw_output: args.trace_raw_output,
            show_thoughts_on_stdout: args.show_thoughts_on_stdout,
            emit_stdout_markdown: args.emit_stdout_markdown,
            iterable_closed_warned: false,
            work_dir: args.work_dir,
            run_timing: args.run_timing,
            session_id: args.session_id,
            deferred_sink,
        }
    }
}

impl Drop for PromptTraceWriter {
    fn drop(&mut self) {
        self.flush_deferred();
    }
}

#[cfg(test)]
pub(crate) async fn open_kpop_timestamp_trace_writer(
    trace_path: &std::path::Path,
) -> PromptTraceWriter {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(trace_path)
        .await
        .unwrap();
    PromptTraceWriter::for_live_prompt(LivePromptTraceArgs {
        file,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        trace_raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        work_dir: PathBuf::new(),
        run_timing: None,
        session_id: String::new(),
    })
}
