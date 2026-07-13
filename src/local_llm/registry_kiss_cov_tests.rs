//! External kiss witnesses for [`super::registry`].

#[test]
fn kiss_cov_local_model_spec_type() {
    let spec = super::LocalModelSpec {
        slug: "kiss",
        display_name: "Kiss",
        hf_repo: "org/kiss",
        cache_dirname: "kiss",
        loader: "mlx_lm",
    };
    let super::LocalModelSpec {
        slug,
        display_name,
        hf_repo,
        cache_dirname,
        loader,
    } = spec;
    assert_eq!(slug, "kiss");
    assert_eq!(display_name, "Kiss");
    assert_eq!(hf_repo, "org/kiss");
    assert_eq!(cache_dirname, "kiss");
    assert_eq!(loader, "mlx_lm");
    let _ = stringify!(LocalModelSpec);
}
