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

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn mbc2_turn(&mut self) -> Result<String, String> {
        Ok("stub mbc2".into())
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

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn mbc2_turn(&mut self) -> Result<String, String> {
        Ok("M".into())
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

    /// # Errors
    ///
    /// Returns `Err` when prompt assembly fails (stub never fails).
    pub fn mbc2_turn(&mut self) -> Result<String, String> {
        Ok("stub mbc2".into())
    }
}

#[cfg(test)]
#[path = "kpop_test_stubs_tests.rs"]
mod kpop_test_stubs_tests;
