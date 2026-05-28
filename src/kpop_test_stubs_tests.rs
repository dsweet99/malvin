use std::sync::{Arc, Mutex};

use crate::kpop_test_stubs::{CaptureWants, EchoPrompts, MtStubPrompts};

#[test]
fn kpop_test_stubs_prompts() {
    let mut mt = MtStubPrompts;
    let _ = mt.kpop_block(1, 0).unwrap();
    let mut echo = EchoPrompts;
    let _ = echo.kpop_block(2, 0).unwrap();
    let wants = Arc::new(Mutex::new(Vec::new()));
    let mut cap = CaptureWants::new(wants.clone());
    let _ = cap.kpop_block(3, 0).unwrap();
    assert_eq!(wants.lock().unwrap().len(), 1);
}
