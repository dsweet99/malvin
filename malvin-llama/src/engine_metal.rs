//! Metal-backed llama.cpp engine (Apple Silicon).

use std::num::NonZeroU32;
use std::path::Path;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;

use crate::chat::ChatTurn;
use crate::engine::CompleteRequest;
use crate::engine_metal_generate as generate;

/// Fixed context window for v1 local completions.
pub const N_CTX: u32 = 8192;

pub struct InnerEngine {
    backend: LlamaBackend,
    model: LlamaModel,
}

impl InnerEngine {
    pub fn load(gguf_path: &Path) -> Result<Self, String> {
        let backend = LlamaBackend::init().map_err(|e| format!("llama backend init: {e}"))?;
        let params = LlamaModelParams::default().with_n_gpu_layers(1_000);
        let model = LlamaModel::load_from_file(&backend, gguf_path, &params)
            .map_err(|e| format!("load GGUF {}: {e}", gguf_path.display()))?;
        Ok(Self { backend, model })
    }

    pub fn complete(&self, request: &CompleteRequest<'_>) -> Result<String, String> {
        let prompt = render_prompt(&self.model, request.turns)?;
        let tokens = tokenize_prompt(&self.model, &prompt)?;
        ensure_prompt_fits_ctx(&tokens)?;
        let mut ctx = open_context(&self.backend, &self.model)?;
        let mut batch = LlamaBatch::new(tokens.len(), 1);
        generate::decode_prompt(&mut generate::DecodePromptArgs {
            ctx: &mut ctx,
            batch: &mut batch,
            tokens: &tokens,
        })?;
        let mut sampler = build_sampler();
        generate::sample_tokens(&mut generate::SampleTokensArgs {
            model: &self.model,
            ctx: &mut ctx,
            batch: &mut batch,
            sampler: &mut sampler,
            max_tokens: request.max_tokens,
        })
    }
}

fn tokenize_prompt(model: &LlamaModel, prompt: &str) -> Result<Vec<LlamaToken>, String> {
    let tokens = model
        .str_to_token(prompt, AddBos::Always)
        .map_err(|e| format!("tokenize: {e}"))?;
    if tokens.is_empty() {
        return Err("local llama prompt tokenized to empty".into());
    }
    Ok(tokens)
}

fn ensure_prompt_fits_ctx(tokens: &[LlamaToken]) -> Result<(), String> {
    if tokens.len() >= N_CTX as usize {
        return Err(format!(
            "local llama prompt has {} tokens; exceeds n_ctx={N_CTX}",
            tokens.len()
        ));
    }
    Ok(())
}

fn open_context<'a>(
    backend: &LlamaBackend,
    model: &'a LlamaModel,
) -> Result<LlamaContext<'a>, String> {
    let n_ctx = NonZeroU32::new(N_CTX).expect("non-zero");
    let ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));
    model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("llama context: {e}"))
}

fn build_sampler() -> LlamaSampler {
    LlamaSampler::chain_simple([
        LlamaSampler::temp(0.7),
        LlamaSampler::top_p(0.9, 1),
        LlamaSampler::dist(1),
    ])
}

pub fn render_prompt(model: &LlamaModel, turns: &[ChatTurn]) -> Result<String, String> {
    let tmpl = model
        .chat_template(None)
        .map_err(|e| format!("chat template: {e}"))?;
    let chat = turns_to_llama(turns)?;
    model
        .apply_chat_template(&tmpl, &chat, true)
        .map_err(|e| format!("apply chat template: {e}"))
}

fn chat_turn_to_llama(turn: &ChatTurn) -> Result<LlamaChatMessage, String> {
    LlamaChatMessage::new(turn.role.clone(), turn.content.clone())
        .map_err(|e| format!("chat message: {e}"))
}

pub fn turns_to_llama(turns: &[ChatTurn]) -> Result<Vec<LlamaChatMessage>, String> {
    turns.iter().map(chat_turn_to_llama).collect()
}

#[cfg(test)]
#[path = "engine_metal_tests.rs"]
mod engine_metal_tests;
