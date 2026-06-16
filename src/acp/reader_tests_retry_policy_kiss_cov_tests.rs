use crate::acp::{
    emit_operational_upgrade_plan_stop, operational_iterable_closed_for_emit, IterableClosedStream,
};

#[test]
fn kiss_cov_emit_operational_upgrade_plan_stop() {
    let mut warned = false;
    emit_operational_upgrade_plan_stop(&mut warned);
    assert!(warned);
}

#[test]
fn kiss_cov_iterable_closed_stream_message() {
    assert_eq!(
        operational_iterable_closed_for_emit("x", Some(IterableClosedStream::Writable)),
        Some("acp: WritableIterable is closed")
    );
    assert_eq!(
        operational_iterable_closed_for_emit("x", Some(IterableClosedStream::Readable)),
        Some("acp: ReadableIterable is closed")
    );
}
