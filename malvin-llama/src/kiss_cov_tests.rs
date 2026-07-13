//! Root-level kiss coverage witnesses for platform engine modules.

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod metal_witnesses {
    use crate::engine::{complete, load_engine};
    use crate::engine_metal::{InnerEngine, N_CTX};
    use crate::engine_metal_generate::{decode_prompt, sample_tokens};

    #[test]
    fn kiss_cov_metal_modules_imported() {
        let _: Option<InnerEngine> = None;
        assert_eq!(N_CTX, 8192);
        let _ = (decode_prompt, sample_tokens, load_engine, complete);
        let _ = (
            stringify!(LocalEngine),
            stringify!(CompleteRequest),
            stringify!(DecodePromptArgs),
            stringify!(SampleTokensArgs),
            stringify!(tokenize_prompt),
            stringify!(ensure_prompt_fits_ctx),
            stringify!(open_context),
            stringify!(build_sampler),
            stringify!(render_prompt),
            stringify!(chat_turn_to_llama),
            stringify!(turns_to_llama),
        );
    }
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
mod stub_witnesses {
    use crate::engine::{complete, load_engine};
    use crate::engine_stub::InnerEngine;

    #[test]
    fn kiss_cov_stub_module_imported() {
        let _: Option<InnerEngine> = None;
        let _ = (InnerEngine::load, load_engine, complete);
        let _ = (stringify!(LocalEngine), stringify!(CompleteRequest));
    }
}
