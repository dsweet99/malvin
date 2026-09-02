use crate::acp::{IterableClosedStream, iterable_closed_stream_from_buffer};

#[test]
fn iterable_closed_stream_from_buffer_detects_both_kinds() {
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: WritableIterable is closed"),
        Some(IterableClosedStream::Writable)
    );
    assert_eq!(
        iterable_closed_stream_from_buffer("Error: T: ReadableIterable is closed"),
        Some(IterableClosedStream::Readable)
    );
    assert_eq!(iterable_closed_stream_from_buffer("ok"), None);
}
