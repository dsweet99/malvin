use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub struct MtStubPrompts;

impl MtStubPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_prompt(&mut self) -> Result<String, String> {
        Ok("stub kpop block".to_string())
    }
}

#[derive(Debug, Default)]
pub struct EchoPrompts;

impl EchoPrompts {
    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_prompt(&mut self) -> Result<String, String> {
        Ok("K".to_string())
    }
}

#[derive(Debug)]
pub struct CaptureBlocks {
    pub blocks: Arc<Mutex<Vec<()>>>,
}

impl CaptureBlocks {
    /// # Panics
    ///
    /// Panics if the blocks mutex is poisoned when recording a block.
    #[must_use]
    pub const fn new(blocks: Arc<Mutex<Vec<()>>>) -> Self {
        Self { blocks }
    }

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn kpop_prompt(&mut self) -> Result<String, String> {
        self.blocks.lock().expect("blocks lock").push(());
        Ok("stub kpop block".to_string())
    }
}

#[cfg(test)]
#[path = "kpop_test_stubs_tests.rs"]
mod kpop_test_stubs_tests;
