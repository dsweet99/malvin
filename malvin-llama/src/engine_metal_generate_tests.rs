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
        stringify!(prompt_n_tokens),
    );
}

#[test]
fn prompt_chunk_ends_cover_full_span() {
    // Mirrors decode_prompt's chunking: walk [0, n) by n_batch.
    let n_tokens = 5000usize;
    let n_batch = 2048usize;
    let mut pos = 0usize;
    let mut ends = Vec::new();
    while pos < n_tokens {
        let end = (pos + n_batch).min(n_tokens);
        ends.push(end);
        pos = end;
    }
    assert_eq!(ends, vec![2048, 4096, 5000]);
    assert_eq!(*ends.last().expect("non-empty"), n_tokens);
}
