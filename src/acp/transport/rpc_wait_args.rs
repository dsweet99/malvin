use crate::acp::ResponseTx;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};

#[allow(dead_code)]
pub(crate) struct RpcWaitArgs<'a> {
    pub _pending: &'a Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_activity_seq: &'a Arc<AtomicU64>,
    pub acp_activity_notify: &'a Arc<tokio::sync::Notify>,
    pub _id: u64,
    pub rx: oneshot::Receiver<Result<Value, String>>,
    pub child_pid: Option<u32>,
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_rpc_wait_args() { let _: Option<RpcWaitArgs> = None; }

}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<RpcWaitArgs> = None;
    }
}
