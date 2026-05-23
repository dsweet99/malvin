#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    use crate::acp::outgoing_prompt_trace::{
        DoPromptTraceSplit, OutgoingPromptTrace, UniformOutgoingTrace,
    };
    include!("session_post_impl.inc");
}

pub(crate) use inline::*;
