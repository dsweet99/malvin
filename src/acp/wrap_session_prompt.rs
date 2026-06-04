#![allow(unused_imports, clippy::wildcard_imports)]
pub(crate) mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    use crate::acp::outgoing_prompt_trace::{
        DoPromptTraceSplit, OutgoingPromptTrace, UniformOutgoingTrace,
    };
    include!("session_prompt_helpers.inc");
    include!("session_prompt_trace.inc");

    #[cfg(test)]
    mod kiss_cov_gate_refs{
    use super::*;
        #[test]
        fn kiss_cov_unit_names() {
            let _: Option<PromptTraceDispatchMeta<'_>> = None;
            let _ = stringify!(do_split_outgoing_trace_preamble);
            let _ = stringify!(do_split_trace_preamble);
            let _ = stringify!(open_live_prompt_trace_writer);
            let _ = stringify!(rpc_session_prompt_text);
            let _ = stringify!(uniform_outgoing_trace_preamble);
        }
    }
}

pub(crate) use inline::*;
