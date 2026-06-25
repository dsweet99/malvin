use super::*;
use tracing::Callsite;
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
fn install_malvin_tracing_warns_when_subscriber_already_set() {
    crate::output::clear_captured_stderr_lines();
    install_malvin_tracing();
    install_malvin_tracing();
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("tracing subscriber already initialized")));
}

#[test]
fn malvin_log_accepts_tracing_level_filters_debug() {
    assert!(malvin_log_accepts_tracing_level(Level::INFO));
    assert!(!malvin_log_accepts_tracing_level(Level::DEBUG));
}

#[test]
fn emit_malvin_tracing_log_routes_by_level() {
    crate::output::clear_captured_stderr_lines();
    emit_malvin_tracing_log(Level::ERROR, "err-level");
    emit_malvin_tracing_log(Level::WARN, "warn-level");
    emit_malvin_tracing_log(Level::INFO, "info-level");
    emit_malvin_tracing_log(Level::INFO, "");
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("err-level")));
    assert!(lines.iter().any(|l| l.contains("warn-level")));
    assert!(lines.iter().any(|l| l.contains("info-level")));
}

#[test]
fn tracing_field_formatting_collects_message_and_custom_fields() {
    assert_eq!(format_debug_tracing_field("message", &"x"), "x");
    assert_eq!(format_debug_tracing_field("k", &"val"), "\"val\"");
    assert_eq!(strip_debug_string_quotes("\"q\""), "q");
    let mut buf = String::new();
    push_log_field(&mut buf, "k", "v");
    assert_eq!(buf, "k=v");
    push_log_field(&mut buf, "message", "hello");
    assert_eq!(buf, "k=v; hello");
}

#[test]
fn log_field_visitor_record_str_and_record_debug_collect_fields() {
    use tracing::field::Visit;

    let cs = tracing::callsite! {
        name: "log_field_visitor_fields",
        kind: tracing::metadata::Kind::EVENT,
        target: "malvin::tracing_init_test",
        level: tracing::Level::INFO,
        fields: message, key,
    };
    let meta = cs.metadata();
    let message = meta.fields().field("message").expect("message");
    let key = meta.fields().field("key").expect("key");

    let mut buf = String::new();
    LogFieldVisitor(&mut buf).record_str(&message, "hello");
    assert_eq!(buf, "hello");
    LogFieldVisitor(&mut buf).record_debug(&key, &"val");
    assert_eq!(buf, "hello key=\"val\"");
}

#[test]
fn subscriber_register_callsite_and_enabled_filter_levels() {
    use tracing::subscriber::Subscriber;

    let info_cs = tracing::callsite! {
        name: "register_callsite_info",
        kind: tracing::metadata::Kind::EVENT,
        target: "malvin::tracing_init_test",
        level: tracing::Level::INFO,
        fields: message,
    };
    let sub = MalvinTracingSubscriber;
    assert!(sub.register_callsite(info_cs.metadata()).is_always());
    assert!(sub.enabled(info_cs.metadata()));

    let debug_cs = tracing::callsite! {
        name: "register_callsite_debug",
        kind: tracing::metadata::Kind::EVENT,
        target: "malvin::tracing_init_test",
        level: tracing::Level::DEBUG,
        fields: message,
    };
    assert!(!sub.enabled(debug_cs.metadata()));
}

#[test]
fn subscriber_event_and_process_malvin_tracing_event_emit_to_stderr() {
    use tracing::field::Value;
    use tracing::subscriber::Subscriber;

    let info_cs = tracing::callsite! {
        name: "process_malvin_tracing_event_info",
        kind: tracing::metadata::Kind::EVENT,
        target: "malvin::tracing_init_test",
        level: tracing::Level::INFO,
        fields: message,
    };
    let info_meta = info_cs.metadata();
    let message = info_meta.fields().field("message").expect("message");
    let values = [(&message, Some(&"direct-event" as &dyn Value))];
    let value_set = info_meta.fields().value_set(&values);
    let event = tracing::Event::new(info_meta, &value_set);
    let sub = MalvinTracingSubscriber;

    crate::output::clear_captured_stderr_lines();
    sub.event(&event);
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("direct-event")));

    crate::output::clear_captured_stderr_lines();
    process_malvin_tracing_event(&event);
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("direct-event")));
}

#[test]
fn subscriber_new_span_record_follows_from_enter_exit_hooks() {
    use tracing::subscriber::Subscriber;

    let sub = MalvinTracingSubscriber;
    let span_cs = tracing::callsite! {
        name: "new_span_hook",
        kind: tracing::metadata::Kind::SPAN,
        target: "malvin::tracing_init_test",
        level: tracing::Level::INFO,
        fields: message,
    };
    let span_meta = span_cs.metadata();
    let span_values = span_meta.fields().value_set(&[]);
    let attrs = tracing::span::Attributes::new(span_meta, &span_values);
    let id = sub.new_span(&attrs);
    assert_eq!(id, tracing::span::Id::from_u64(1));
    sub.record_follows_from(&id, &tracing::span::Id::from_u64(2));
    sub.enter(&id);
    sub.exit(&id);
}

#[test]
fn process_malvin_tracing_event_emits_info_not_debug() {
    use tracing::dispatcher::Dispatch;

    crate::output::clear_captured_stderr_lines();
    let dispatch = Dispatch::new(MalvinTracingSubscriber);
    tracing::dispatcher::with_default(&dispatch, || {
        tracing::info!(target: "malvin::process_malvin_tracing_event_test", "info-visible");
        tracing::debug!(target: "malvin::process_malvin_tracing_event_test", "debug-hidden");
    });
    let lines = crate::output::take_captured_stderr_lines();
    assert!(lines.iter().any(|l| l.contains("info-visible")));
    assert!(!lines.iter().any(|l| l.contains("debug-hidden")));
}

#[test]
fn span_lifecycle_does_not_change_event_stderr_output() {
    use tracing::dispatcher::Dispatch;

    const MSG: &str = "malvin-tracing-invariance-msg";

    fn event_only_lines() -> Vec<String> {
        crate::output::clear_captured_stderr_lines();
        let dispatch = Dispatch::new(MalvinTracingSubscriber);
        tracing::dispatcher::with_default(&dispatch, || {
            tracing::info!(target: "malvin::invariance_test", "{MSG}");
        });
        crate::output::take_captured_stderr_lines()
    }

        #[allow(unused_variables)]
        fn span_heavy_lines() -> Vec<String> {
            crate::output::clear_captured_stderr_lines();
            let dispatch = Dispatch::new(MalvinTracingSubscriber);
            tracing::dispatcher::with_default(&dispatch, || {
                let root = tracing::info_span!("malvin_invariance_root");
                let mut current = root;
                for i in 0..32 {
                    current = tracing::info_span!(parent: &current, "malvin_invariance_depth_{i}");
                }
            current.follows_from(&tracing::info_span!("malvin_invariance_follows"));
            let _entered = current.enter();
            tracing::info!(target: "malvin::invariance_test", "{MSG}");
        });
        crate::output::take_captured_stderr_lines()
    }

    let event_lines: Vec<_> = event_only_lines()
        .into_iter()
        .filter(|l| l.contains(MSG))
        .collect();
    let span_lines: Vec<_> = span_heavy_lines()
        .into_iter()
        .filter(|l| l.contains(MSG))
        .collect();
    assert_eq!(event_lines, span_lines);
    assert_eq!(event_lines.len(), 1);
}
