use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(install_malvin_tracing);
}

fn install_malvin_tracing() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    let subscriber = tracing_subscriber::registry().with(MalvinLogLayer);
    if subscriber.try_init().is_err() {
        crate::output::print_log_warning(
            "tracing subscriber already initialized; malvin log layer not installed",
        );
    }
}

#[must_use]
pub(crate) fn malvin_log_accepts_tracing_level(level: tracing::Level) -> bool {
    level <= tracing::Level::INFO
}

struct MalvinLogLayer;

impl<S> tracing_subscriber::Layer<S> for MalvinLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let level = *event.metadata().level();
        if !malvin_log_accepts_tracing_level(level) {
            return;
        }
        let mut msg = String::new();
        event.record(&mut LogFieldVisitor(&mut msg));
        if msg.is_empty() {
            return;
        }
        match level {
            tracing::Level::ERROR => crate::output::print_log_error(&msg),
            tracing::Level::WARN => crate::output::print_log_warning(&msg),
            _ => crate::output::print_stderr_line(crate::output::MALVIN_WHO, &msg),
        }
    }
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
mod level_tests {
    use super::malvin_log_accepts_tracing_level;
    use tracing::Level;
    use tracing::field::Visit;
    use tracing_subscriber::Layer;

    #[test]
    fn tracing_level_order_matches_malvin_log_filter() {
        assert!(Level::WARN <= Level::INFO);
        assert!(Level::ERROR <= Level::INFO);
        assert!(Level::DEBUG > Level::INFO);
        assert!(malvin_log_accepts_tracing_level(Level::INFO));
        assert!(malvin_log_accepts_tracing_level(Level::WARN));
        assert!(malvin_log_accepts_tracing_level(Level::ERROR));
        assert!(!malvin_log_accepts_tracing_level(Level::DEBUG));
    }

    #[test]
    fn on_event_routes_info_to_stderr_capture() {
        let _: Option<super::MalvinLogLayer> = None;
        let _: Option<super::LogFieldVisitor<'_>> = None;
        let _ = <super::MalvinLogLayer as Layer<tracing_subscriber::Registry>>::on_event;
        super::init_tracing();
        crate::output::clear_captured_stderr_lines();
        tracing::info!(target: "malvin::tracing_init_test", "on-event-smoke");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(lines.iter().any(|l| l.contains("on-event-smoke")));
    }

    #[test]
    fn on_event_routes_warn_and_error() {
        super::init_tracing();
        crate::output::clear_captured_stderr_lines();
        tracing::warn!(target: "malvin::tracing_init_test", "warn-smoke");
        tracing::error!(target: "malvin::tracing_init_test", "err-smoke");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(lines.iter().any(|l| l.contains("warn-smoke")));
        assert!(lines.iter().any(|l| l.contains("err-smoke")));
    }

    #[test]
    fn on_event_ignores_debug_level() {
        super::init_tracing();
        crate::output::clear_captured_stderr_lines();
        tracing::debug!(target: "malvin::tracing_init_test", "debug-hidden");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(!lines.iter().any(|l| l.contains("debug-hidden")));
    }

    #[test]
    fn on_event_formats_structured_fields_via_log_field_visitor() {
        let _ = <super::LogFieldVisitor as Visit>::record_str;
        let _ = <super::LogFieldVisitor as Visit>::record_debug;
        super::init_tracing();
        crate::output::clear_captured_stderr_lines();
        tracing::info!(target: "malvin::tracing_init_test", code = 42u32, "with-fields");
        let lines = crate::output::take_captured_stderr_lines();
        let joined = lines.join("\n");
        assert!(joined.contains("with-fields"));
        assert!(joined.contains("code="));
    }

    #[test]
    fn push_log_field_formats_message_and_kv() {
        let mut buf = String::new();
        super::push_log_field(&mut buf, "message", "msg");
        super::push_log_field(&mut buf, "extra", "v");
        assert_eq!(buf, "msg extra=v");
    }
}
