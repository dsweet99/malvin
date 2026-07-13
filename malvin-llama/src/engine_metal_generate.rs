//! Prompt decode + token sampling helpers for the Metal engine.

use encoding_rs::UTF_8;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

pub struct DecodePromptArgs<'a, 'ctx> {
    pub ctx: &'a mut LlamaContext<'ctx>,
    pub batch: &'a mut LlamaBatch<'static>,
    pub tokens: &'a [LlamaToken],
}

pub struct SampleTokensArgs<'a, 'ctx> {
    pub model: &'a LlamaModel,
    pub ctx: &'a mut LlamaContext<'ctx>,
    pub batch: &'a mut LlamaBatch<'static>,
    pub sampler: &'a mut LlamaSampler,
    pub max_tokens: i32,
    /// Absolute KV position after the prompt (full prompt length, not last chunk size).
    pub prompt_n_tokens: i32,
}

/// Decode the full prompt in chunks of `ctx.n_batch()` so prompts larger than the
/// default llama.cpp `n_batch` (2048) do not trip `n_tokens_all <= n_batch`.
pub fn decode_prompt(args: &mut DecodePromptArgs<'_, '_>) -> Result<(), String> {
    let n_batch = args.ctx.n_batch().max(1) as usize;
    let last = args.tokens.len() - 1;
    let mut pos = 0usize;
    while pos < args.tokens.len() {
        args.batch.clear();
        let end = (pos + n_batch).min(args.tokens.len());
        for (offset, token) in args.tokens[pos..end].iter().enumerate() {
            let i = pos + offset;
            args.batch
                .add(
                    *token,
                    i32::try_from(i).unwrap_or(0),
                    &[0],
                    i == last,
                )
                .map_err(|e| format!("batch add: {e}"))?;
        }
        args.ctx
            .decode(args.batch)
            .map_err(|e| format!("llama_decode prompt: {e}"))?;
        pos = end;
    }
    Ok(())
}

pub fn sample_tokens(args: &mut SampleTokensArgs<'_, '_>) -> Result<String, String> {
    let mut decoder = UTF_8.new_decoder();
    let mut out = String::new();
    // Absolute sequence position after the full prompt (not last-chunk batch size).
    let mut n_pos = args.prompt_n_tokens;
    for _ in 0..args.max_tokens.max(1) {
        // llama_sampler_sample idx is the logits row (-1 = last), not KV position.
        let token = args.sampler.sample(args.ctx, -1);
        args.sampler.accept(token);
        if args.model.is_eog_token(token) {
            break;
        }
        let piece = args
            .model
            .token_to_piece(token, &mut decoder, true, None)
            .map_err(|e| format!("detokenize: {e}"))?;
        out.push_str(&piece);
        args.batch.clear();
        args.batch
            .add(token, n_pos, &[0], true)
            .map_err(|e| format!("batch add: {e}"))?;
        args.ctx
            .decode(args.batch)
            .map_err(|e| format!("llama_decode step: {e}"))?;
        n_pos += 1;
    }
    Ok(out)
}

#[cfg(test)]
#[path = "engine_metal_generate_tests.rs"]
mod engine_metal_generate_tests;
