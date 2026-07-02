use std::sync::{Arc, Mutex};

use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};

#[test]
fn kpop_test_stubs_prompts() {
    let mut mt = MtStubPrompts;
    let _ = mt.kpop_block().unwrap();
    let mut echo = EchoPrompts;
    let _ = echo.kpop_block().unwrap();
    let blocks = Arc::new(Mutex::new(Vec::new()));
    let mut cap = CaptureBlocks::new(blocks.clone());
    let _ = cap.kpop_block().unwrap();
    assert_eq!(blocks.lock().unwrap().len(), 1);
}
