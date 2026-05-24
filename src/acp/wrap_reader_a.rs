#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    include!("reader_inline.rs");
    include!("reader_stdout_body_a.inc");
}

pub(crate) use inline::*;

#[cfg(test)]
#[path = "reader_inline_tests.rs"]
mod reader_inline_tests;
