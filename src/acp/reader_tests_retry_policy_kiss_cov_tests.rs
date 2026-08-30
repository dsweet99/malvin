use crate::acp::{IterableClosedStream, iterable_closed_stream_from_buffer};

#[test]
fn kiss_cov_iterable_closed_stream_from_buffer_kinds() {
    assert_eq!(
        iterable_closed_stream_from_buffer("WritableIterable is closed"),
        Some(IterableClosedStream::Writable)
    );
    assert_eq!(
        iterable_closed_stream_from_buffer("ReadableIterable is closed"),
        Some(IterableClosedStream::Readable)
    );
}
