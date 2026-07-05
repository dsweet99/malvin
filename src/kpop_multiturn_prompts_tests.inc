use crate::kpop_multiturn_prompts::{KpopMultiturnPrompts, SmokeKpopBuilder};
use crate::kpop_test_stubs::{CaptureBlocks, EchoPrompts, MtStubPrompts};

fn assert_all_phases(p: &mut KpopMultiturnPrompts<'_>) {
    let _ = p.kpop_priors().unwrap();
    let _ = p.kpop_block().unwrap();
    let _ = p.kpop_block_a().unwrap();
    let _ = p.kpop_block_b().unwrap();
    let _ = p.kpop_block_c().unwrap();
}

#[test]
fn kpop_multiturn_stub_mt_phases() {
    assert_all_phases(&mut KpopMultiturnPrompts::StubMt(MtStubPrompts));
}

#[test]
fn kpop_multiturn_stub_echo_phases() {
    assert_all_phases(&mut KpopMultiturnPrompts::StubEcho(EchoPrompts));
}

#[test]
fn kpop_multiturn_stub_capture_phases() {
    let blocks = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    assert_all_phases(&mut KpopMultiturnPrompts::StubCapture(CaptureBlocks::new(blocks)));
}

#[test]
fn kpop_multiturn_smoke_phases() {
    assert_all_phases(&mut KpopMultiturnPrompts::Smoke(SmokeKpopBuilder));
}
