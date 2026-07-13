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
}

pub fn decode_prompt(args: &mut DecodePromptArgs<'_, '_>) -> Result<(), String> {
    args.batch.clear();
    let last = args.tokens.len() - 1;
    for (i, token) in args.tokens.iter().enumerate() {
        args.batch
            .add(*token, i32::try_from(i).unwrap_or(0), &[0], i == last)
            .map_err(|e| format!("batch add: {e}"))?;
    }
    args.ctx
        .decode(args.batch)
        .map_err(|e| format!("llama_decode prompt: {e}"))
}

pub fn sample_tokens(args: &mut SampleTokensArgs<'_, '_>) -> Result<String, String> {
    let mut decoder = UTF_8.new_decoder();
    let mut out = String::new();
    // Sequence position after the prompt batch (not the logits-row index).
    let mut n_pos = args.batch.n_tokens();
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
