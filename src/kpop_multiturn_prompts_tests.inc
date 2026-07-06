use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};

fn assert_kpop_prompt(p: &mut KpopMultiturnPrompts<'_>) {
    let _ = p.kpop_prompt().unwrap();
}

#[test]
fn kpop_multiturn_stub_mt_phases() {
    assert_kpop_prompt(&mut KpopMultiturnPrompts::StubMt(MtStubPrompts));
}

#[test]
fn kpop_multiturn_stub_echo_phases() {
    assert_kpop_prompt(&mut KpopMultiturnPrompts::StubEcho(EchoPrompts));
}

#[test]
fn kpop_multiturn_stub_capture_phases() {
    let blocks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    assert_kpop_prompt(&mut KpopMultiturnPrompts::StubCapture(CaptureBlocks::new(blocks)));
}

#[test]
fn kpop_multiturn_smoke_phases() {
    assert_kpop_prompt(&mut KpopMultiturnPrompts::Smoke(SmokeKpopBuilder));
}
