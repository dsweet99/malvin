#![allow(unused_imports, clippy::wildcard_imports)]
mod inline {
    use crate::acp::import_prelude::*;
    use crate::acp::*;
    include!("retry_policy.rs");
}

#[path = "retry_teardown.rs"]
mod retry_teardown;

pub(crate) use inline::*;
pub(crate) use retry_teardown::*;

#[cfg(test)]
#[path = "retry_policy_test_mods.rs"]
mod retry_policy_test_mods;
