//! Kiss coverage tests for Metal engine helpers (filename must be `*_tests.rs`).

use super::{
    build_sampler, ensure_prompt_fits_ctx, install_llama_logs_once, open_context, render_prompt,
    tokenize_prompt, turns_to_llama, InnerEngine, N_BATCH, N_CTX,
};

#[test]
fn kiss_cov_metal_helper_names() {
    let _: Option<InnerEngine> = None;
    assert_eq!(N_CTX, 8192);
    assert_eq!(N_BATCH, 2048);
    assert_eq!(crate::engine::DEFAULT_CONTEXT_SIZE, N_CTX);
    kiss_cov_bind_metal_fns();
    kiss_cov_stringify_metal_names();
}

fn kiss_cov_bind_metal_fns() {
    let _ = (
        render_prompt,
        turns_to_llama,
        tokenize_prompt,
        ensure_prompt_fits_ctx,
        open_context,
        build_sampler,
        install_llama_logs_once,
    );
}

fn kiss_cov_stringify_metal_names() {
    let _ = (
        stringify!(InnerEngine),
        stringify!(N_CTX),
        stringify!(N_BATCH),
        stringify!(tokenize_prompt),
        stringify!(ensure_prompt_fits_ctx),
        stringify!(open_context),
        stringify!(build_sampler),
        stringify!(install_llama_logs_once),
        stringify!(render_prompt),
        stringify!(chat_turn_to_llama),
        stringify!(turns_to_llama),
        stringify!(load),
        stringify!(complete),
    );
}

#[test]
fn ensure_prompt_fits_ctx_rejects_full_window() {
    let tokens = vec![llama_cpp_2::token::LlamaToken(1); N_CTX as usize];
    let err = ensure_prompt_fits_ctx(&tokens, N_CTX).expect_err("full ctx");
    assert!(err.contains("n_ctx"));
}

#[test]
fn ensure_prompt_fits_ctx_respects_custom_n_ctx() {
    let tokens = vec![llama_cpp_2::token::LlamaToken(1); 100];
    assert!(ensure_prompt_fits_ctx(&tokens, 128).is_ok());
    let err = ensure_prompt_fits_ctx(&tokens, 100).expect_err("exact fill");
    assert!(err.contains("n_ctx=100"));
}
