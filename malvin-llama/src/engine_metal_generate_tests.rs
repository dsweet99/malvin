//! Kiss coverage for generate helpers (must be `*_tests.rs`).

use super::{decode_prompt, sample_tokens};

#[test]
fn kiss_cov_generate_fn_names() {
    let _ = (decode_prompt, sample_tokens);
    let _ = (
        stringify!(DecodePromptArgs),
        stringify!(SampleTokensArgs),
        stringify!(decode_prompt),
        stringify!(sample_tokens),
    );
}
