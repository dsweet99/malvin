//! JSONL bridge protocol types (shared with Cursor; optional provider fields).

#[allow(unused_imports)]
pub use crate::bridge_protocol::{
    decode_event as prime_decode_event, encode_request as prime_encode_request,
    BridgeEvent as PrimeBridgeEvent, BridgeRequest as PrimeBridgeRequest,
};
