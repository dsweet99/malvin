use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};

#[test]
fn kpop_multiturn_prompts_variants() {
    let mut turn = KpopMultiturnPrompts::StubMt(MtStubPrompts);
    let _ = turn.kpop_block().unwrap();
    let mut echo = KpopMultiturnPrompts::StubEcho(EchoPrompts);
    let _ = echo.kpop_block().unwrap();
    let blocks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut cap = KpopMultiturnPrompts::StubCapture(CaptureBlocks::new(blocks));
    let _ = cap.kpop_block().unwrap();
    let mut smoke = KpopMultiturnPrompts::Smoke(SmokeKpopBuilder);
    let _ = smoke.kpop_block().unwrap();
}
