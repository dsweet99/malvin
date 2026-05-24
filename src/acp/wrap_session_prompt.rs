#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    use crate::acp::outgoing_prompt_trace::{
        DoPromptTraceSplit, OutgoingPromptTrace, UniformOutgoingTrace,
    };
    include!("session_prompt_helpers.inc");
    include!("session_prompt_trace.inc");
}

pub(crate) use inline::*;
