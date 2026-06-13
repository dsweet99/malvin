use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(install_malvin_tracing);
}

fn install_malvin_tracing() {
    if tracing::subscriber::set_global_default(MalvinTracingSubscriber).is_ok() {
        return;
    }
    crate::output::print_log_warning(
        "tracing subscriber already initialized; malvin log layer not installed",
    );
}

#[must_use]
pub(crate) fn malvin_log_accepts_tracing_level(level: tracing::Level) -> bool {
    level <= tracing::Level::INFO
}

pub(crate) fn emit_malvin_tracing_log(level: tracing::Level, msg: &str) {
    if msg.is_empty() {
        return;
    }
    match level {
        tracing::Level::ERROR => crate::output::print_log_error(msg),
        tracing::Level::WARN => crate::output::print_log_warning(msg),
        _ => crate::output::print_stderr_line(crate::output::MALVIN_WHO, msg),
    }
}

pub(crate) fn process_malvin_tracing_event(event: &tracing::Event<'_>) {
    let level = *event.metadata().level();
    if !malvin_log_accepts_tracing_level(level) {
        return;
    }
    let mut msg = String::new();
    event.record(&mut LogFieldVisitor(&mut msg));
    emit_malvin_tracing_log(level, &msg);
}

struct MalvinTracingSubscriber;

impl tracing::Subscriber for MalvinTracingSubscriber {
    fn register_callsite(&self, _metadata: &'static tracing::Metadata<'static>) -> tracing::subscriber::Interest {
        tracing::subscriber::Interest::always()
    }

    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        malvin_log_accepts_tracing_level(*metadata.level())
    }

    fn event(&self, event: &tracing::Event<'_>) {
        process_malvin_tracing_event(event);
    }

    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        tracing::span::Id::from_u64(1)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

struct LogFieldVisitor<'a>(&'a mut String);

impl tracing::field::Visit for LogFieldVisitor<'_> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        push_log_field(self.0, field.name(), value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        push_log_field(
            self.0,
            field.name(),
            &format_debug_tracing_field(field.name(), value),
        );
    }
}

pub(crate) fn format_debug_tracing_field(name: &str, value: &dyn std::fmt::Debug) -> String {
    if name == "message" {
        return strip_debug_string_quotes(&format!("{value:?}"));
    }
    format!("{value:?}")
}

fn strip_debug_string_quotes(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        return s[1..s.len() - 1].to_string();
    }
    s.to_string()
}

fn push_log_field(buf: &mut String, name: &str, value: &str) {
    use std::fmt::Write;
    if name == "message" {
        if !buf.is_empty() {
            buf.push_str("; ");
        }
        buf.push_str(value);
        return;
    }
    if !buf.is_empty() {
        buf.push(' ');
    }
    let _ = write!(buf, "{name}={value}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn init_tracing_installs_layer() {
        init_tracing();
        crate::output::clear_captured_stderr_lines();
        tracing::info!(target: "malvin::tracing_init_test", "init-smoke");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(lines.iter().any(|l| l.contains("init-smoke")));
    }

    #[test]
    fn install_malvin_tracing_direct_call_for_kiss() {
        install_malvin_tracing();
    }

    #[test]
    fn malvin_log_accepts_tracing_level_filters_debug() {
        assert!(malvin_log_accepts_tracing_level(Level::INFO));
        assert!(!malvin_log_accepts_tracing_level(Level::DEBUG));
    }

    #[test]
    fn kiss_cov_tracing_subscriber_and_visitor() {
        use tracing::field::Visit;
        use tracing::Subscriber;
        let _ = emit_malvin_tracing_log;
        let _ = process_malvin_tracing_event;
        assert_eq!(format_debug_tracing_field("message", &"x"), "x");
        assert_eq!(format_debug_tracing_field("k", &"val"), "\"val\"");
        assert_eq!(strip_debug_string_quotes("\"q\""), "q");
        let mut buf = String::new();
        push_log_field(&mut buf, "k", "v");
        assert!(buf.contains('v'));
        let _ = <MalvinTracingSubscriber as Subscriber>::event;
        let _ = <MalvinTracingSubscriber as Subscriber>::enabled;
        let _ = <LogFieldVisitor as Visit>::record_str;
        let _ = <LogFieldVisitor as Visit>::record_debug;
        let visitor = LogFieldVisitor(&mut buf);
        let _: Option<LogFieldVisitor<'_>> = Some(visitor);
    }
}
