#![allow(unused_imports, clippy::wildcard_imports)]
pub(crate) mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    include!("reader_stdout_body_b.inc");

}

pub(crate) use inline::*;

