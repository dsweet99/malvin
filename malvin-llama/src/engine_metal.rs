//! Metal-backed llama.cpp engine (Apple Silicon).

use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Once;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaChatMessage, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use llama_cpp_2::token::LlamaToken;
use llama_cpp_2::send_logs_to_tracing;

use crate::chat::ChatTurn;
use crate::engine::CompleteRequest;

/// Default context window for v1 local completions (also the config.toml default).
pub const N_CTX: u32 = 8192;
const _: () = assert!(N_CTX == crate::engine::DEFAULT_CONTEXT_SIZE);

/// Logical decode batch size (llama.cpp default). Prompts longer than this are
/// submitted in chunks by [`crate::engine_metal_generate::decode_prompt`]; keeping
/// the default avoids growing logits/embedding buffers with `n_ctx` (important for
/// Qwen under USS caps).
pub const N_BATCH: u32 = 2048;

pub struct InnerEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    n_ctx: u32,
}

struct OpenContextArgs<'a> {
    backend: &'a LlamaBackend,
    model: &'a LlamaModel,
    n_ctx: u32,
}

impl InnerEngine {
    pub fn load(gguf_path: &Path, n_ctx: u32) -> Result<Self, String> {
        install_llama_logs_once();
        let n_ctx = NonZeroU32::new(n_ctx)
            .ok_or_else(|| "context_size must be positive".to_string())?
            .get();
        let backend = LlamaBackend::init().map_err(|e| format!("llama backend init: {e}"))?;
        let params = LlamaModelParams::default().with_n_gpu_layers(1_000);
        let model = LlamaModel::load_from_file(&backend, gguf_path, &params)
            .map_err(|e| format!("load GGUF {}: {e}", gguf_path.display()))?;
        Ok(Self {
            backend,
            model,
            n_ctx,
        })
    }

    pub fn complete(&self, request: &CompleteRequest<'_>) -> Result<String, String> {
        let prompt = render_prompt(&self.model, request.turns)?;
        let tokens = tokenize_prompt(&self.model, &prompt)?;
        ensure_prompt_fits_ctx(&tokens, self.n_ctx)?;
        let mut ctx = open_context(&OpenContextArgs {
            backend: &self.backend,
            model: &self.model,
            n_ctx: self.n_ctx,
        })?;
        let mut batch = LlamaBatch::new(N_BATCH as usize, 1);
        crate::engine_metal_generate::decode_prompt(
            &mut crate::engine_metal_generate::DecodePromptArgs {
                ctx: &mut ctx,
                batch: &mut batch,
                tokens: &tokens,
            },
        )?;
        let mut sampler = build_sampler();
        let prompt_n_tokens = i32::try_from(tokens.len()).unwrap_or(i32::MAX);
        crate::engine_metal_generate::sample_tokens(
            &mut crate::engine_metal_generate::SampleTokensArgs {
                model: &self.model,
                ctx: &mut ctx,
                batch: &mut batch,
                sampler: &mut sampler,
                max_tokens: request.max_tokens,
                prompt_n_tokens,
            },
        )
    }
}

fn install_llama_logs_once() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        // Route llama.cpp / ggml logs into `tracing`; malvin's subscriber writes them
        // through the malvin logger (not raw stderr).
        send_logs_to_tracing(llama_cpp_2::LogOptions::default());
    });
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

fn ensure_prompt_fits_ctx(tokens: &[LlamaToken], n_ctx: u32) -> Result<(), String> {
    if tokens.len() >= n_ctx as usize {
        return Err(format!(
            "local llama prompt has {} tokens; exceeds n_ctx={n_ctx}",
            tokens.len()
        ));
    }
    Ok(())
}

fn open_context<'a>(args: &OpenContextArgs<'a>) -> Result<LlamaContext<'a>, String> {
    let n_ctx =
        NonZeroU32::new(args.n_ctx).ok_or_else(|| "context_size must be positive".to_string())?;
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(Some(n_ctx))
        .with_n_batch(N_BATCH);
    args.model
        .new_context(args.backend, ctx_params)
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
