use std::sync::Mutex;

use super::bridge_sdk::child_process_session::ChildProcessSession;
use super::bridge_sdk::stdio_child::StdioChild;
use super::bridge_sdk::stream_log::StreamLog;

pub struct BridgeSession {
    pub stdio: StdioChild,
    pub agent_id: Mutex<Option<String>>,
}

impl std::ops::Deref for BridgeSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.stdio.log
    }
}

impl std::ops::DerefMut for BridgeSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdio.log
    }
}

impl ChildProcessSession for BridgeSession {
    fn stdio(&self) -> &StdioChild {
        &self.stdio
    }

    fn stdio_mut(&mut self) -> &mut StdioChild {
        &mut self.stdio
    }
}
