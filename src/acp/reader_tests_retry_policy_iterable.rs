use crate::acp::{
    IterableClosedStream, iterable_closed_stream_from_buffer,
    operational_iterable_closed_for_emit, operational_iterable_closed_log_line,
};

#[test]
fn iterable_closed_stream_from_buffer_and_operational_iterable_closed_for_emit() {
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: WritableIterable is closed"),
        Some(IterableClosedStream::Writable)
    );
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: ReadableIterable is closed"),
        Some(IterableClosedStream::Readable)
    );
    assert_eq!(
        operational_iterable_closed_for_emit("partial", Some(IterableClosedStream::Writable)),
        Some("acp: WritableIterable is closed")
    );
    assert_eq!(
        operational_iterable_closed_for_emit("partial", Some(IterableClosedStream::Readable)),
        Some("acp: ReadableIterable is closed")
    );
    assert_eq!(operational_iterable_closed_for_emit("ok", None), None);
}

#[test]
fn operational_iterable_closed_log_line_detection() {
    assert_eq!(
        operational_iterable_closed_log_line("\n\nError: T: WritableIterable is closed"),
        Some("acp: WritableIterable is closed")
    );
    assert_eq!(
        operational_iterable_closed_log_line("ReadableIterable is closed"),
        Some("acp: ReadableIterable is closed")
    );
    assert_eq!(operational_iterable_closed_log_line("invalid json"), None);
}

#[test]
fn operational_iterable_closed_for_emit_uses_stream_kind_message() {
    assert_eq!(
        operational_iterable_closed_for_emit(
            "partial",
            Some(IterableClosedStream::Writable)
        ),
        Some("acp: WritableIterable is closed")
    );
}
