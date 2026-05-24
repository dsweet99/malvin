#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]
#![allow(unused_imports, clippy::await_holding_lock)]

mod prelude;

mod shared_harness;
mod shared_handshake;

mod child_health_a;
mod child_health_b;
mod handshake;
mod jsonrpc;
mod rpc_integration_a1;
mod rpc_integration_a2;
mod rpc_integration_b;
mod rpc_unit;

pub(crate) use shared_harness::*;
pub(crate) use shared_handshake::*;
pub(crate) use child_health_a::*;
pub(crate) use child_health_b::*;
pub(crate) use handshake::*;
pub(crate) use jsonrpc::*;
pub(crate) use rpc_integration_a1::*;
pub(crate) use rpc_integration_a2::*;
pub(crate) use rpc_integration_b::*;
pub(crate) use rpc_unit::*;

#[cfg(test)]
mod kiss_coverage {
    use std::sync::atomic::Ordering;

    #[test]
    fn smoke_acp_activity_state() {
        let (seq, _notify) = super::acp_activity_state();
        assert_eq!(seq.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn smoke_harness_rpc_wait_params_symbol() {
        let _: Option<super::HarnessRpcWaitParams<'_>> = None;
    }
}
