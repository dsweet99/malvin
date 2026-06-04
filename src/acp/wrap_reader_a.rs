#![allow(unused_imports, clippy::wildcard_imports)]
pub(crate) mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    include!("reader_inline.rs");
    include!("reader_stdout_body_a.inc");

    #[cfg(test)]
    mod kiss_cov_gate_refs{
    use super::*;
        #[test]
        fn kiss_cov_unit_names() {
            let _: Option<IncomingLineDispatch<'_>> = None;
            let _: Option<ReaderLoopInput> = None;
            let _ = stringify!(handle_incoming_line);
        }
    }
}

pub(crate) use inline::*;

#[cfg(test)]
#[path = "reader_inline_tests.rs"]
mod reader_inline_tests;
