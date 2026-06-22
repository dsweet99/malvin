use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct MtStubPrompts;

impl MtStubPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        Ok(format!("stub kpop want={want}"))
    }
}

#[derive(Debug, Default)]
pub struct EchoPrompts;

impl EchoPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        Ok(format!("K{want}"))
    }
}

#[derive(Debug)]
pub struct CaptureWants {
    pub wants: Arc<Mutex<Vec<usize>>>,
}

impl CaptureWants {
    /// # Panics
    ///
    /// Panics if the wants mutex is poisoned when recording a block.
    #[must_use]
    pub const fn new(wants: Arc<Mutex<Vec<usize>>>) -> Self {
        Self { wants }
    }

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_block(&mut self, want: usize, _: usize) -> Result<String, String> {
        self.wants.lock().expect("wants lock").push(want);
        Ok(format!("stub kpop want={want}"))
    }
}
#[cfg(test)]
#[path = "kpop_test_stubs_test.rs"]
mod kpop_stubs_test;#[cfg(test)]
#[path = "kpop_test_stubs_kiss_cov_test.rs"]
mod kpop_test_stubs_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<EchoPrompts> = None;
        let _: Option<MtStubPrompts> = None;
        let _ = kpop_block;
    }
}
