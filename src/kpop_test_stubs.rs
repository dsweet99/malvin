use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct MtStubPrompts;

impl MtStubPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_block(&mut self, max_hypotheses: usize) -> Result<String, String> {
        Ok(format!("stub kpop max_hypotheses={max_hypotheses}"))
    }
}

#[derive(Debug, Default)]
pub struct EchoPrompts;

impl EchoPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_block(&mut self, max_hypotheses: usize) -> Result<String, String> {
        Ok(format!("K{max_hypotheses}"))
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
    pub fn kpop_block(&mut self, max_hypotheses: usize) -> Result<String, String> {
        self.wants.lock().expect("wants lock").push(max_hypotheses);
        Ok(format!("stub kpop max_hypotheses={max_hypotheses}"))
    }
}

#[cfg(test)]
#[path = "kpop_test_stubs_tests.rs"]
mod kpop_test_stubs_tests;
