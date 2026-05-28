use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_test_stubs::{CaptureWants, EchoPrompts, MtStubPrompts};

#[test]
fn kpop_multiturn_prompts_variants() {
    let mut turn = KpopMultiturnPrompts::StubMt(MtStubPrompts);
    let _ = turn.kpop_block(1, 0).unwrap();
    let mut echo = KpopMultiturnPrompts::StubEcho(EchoPrompts);
    let _ = echo.kpop_block(1, 0).unwrap();
    let wants = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut cap = KpopMultiturnPrompts::StubCapture(CaptureWants::new(wants));
    let _ = cap.kpop_block(2, 0).unwrap();
    let mut smoke = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
    let _ = smoke.kpop_block(1, 0).unwrap();
}
