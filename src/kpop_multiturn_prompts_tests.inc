use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_test_stubs::{CaptureWants, EchoPrompts, MtStubPrompts};

fn assert_kpop_block(p: &mut KpopMultiturnPrompts<'_>) {
    let _ = p.kpop_block(1).unwrap();
}

#[test]
fn kpop_multiturn_stub_mt_phases() {
    assert_kpop_block(&mut KpopMultiturnPrompts::StubMt(MtStubPrompts));
}

#[test]
fn kpop_multiturn_stub_echo_phases() {
    assert_kpop_block(&mut KpopMultiturnPrompts::StubEcho(EchoPrompts));
}

#[test]
fn kpop_multiturn_stub_capture_phases() {
    let wants = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    assert_kpop_block(&mut KpopMultiturnPrompts::StubCapture(CaptureWants::new(wants)));
}

#[test]
fn kpop_multiturn_smoke_phases() {
    assert_kpop_block(&mut KpopMultiturnPrompts::Smoke(SmokeKpopBuilder));
}
