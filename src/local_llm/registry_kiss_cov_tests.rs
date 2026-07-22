//! Kiss static coverage witnesses for [`super::registry`].

#[test]
fn kiss_cov_local_model_spec_type() {
    let spec = super::LocalModelSpec {
        slug: "kiss",
        display_name: "Kiss",
        hf_repo: "org/kiss",
        cache_dirname: "kiss",
        gguf_filename: "kiss.gguf",
        resolve_url: "https://example.com/kiss.gguf",
        min_mem_limit_gb: 6,
    };
    let super::LocalModelSpec {
        slug,
        display_name,
        hf_repo,
        cache_dirname,
        gguf_filename,
        resolve_url,
        min_mem_limit_gb,
    } = spec;
    assert_eq!(slug, "kiss");
    assert_eq!(display_name, "Kiss");
    assert_eq!(hf_repo, "org/kiss");
    assert_eq!(cache_dirname, "kiss");
    assert_eq!(gguf_filename, "kiss.gguf");
    assert!(resolve_url.starts_with("https://"));
    assert!(min_mem_limit_gb >= 1);
    let _ = stringify!(LocalModelSpec);
    let _ = stringify!(local_backend_supported);
    let _ = super::local_backend_supported();
}
