use super::discover::catalog::{CatalogChild, reap_catalog_child, spawn_codex_model_server};

#[test]
fn kiss_cov_catalog_names() {
    let _ = super::discover::catalog::codex_list_models_timeout();
    let _ = spawn_codex_model_server();
}

#[cfg(unix)]
#[test]
fn catalog_wrap_drop_and_reap() {
    let child = std::process::Command::new("true").spawn().unwrap();
    let mut wrapped = CatalogChild::wrap(child);
    reap_catalog_child(&mut wrapped.child);
    drop(wrapped);
}

#[test]
fn catalog_timeout_helper_is_positive() {
    assert!(super::discover::catalog::codex_list_models_timeout() > std::time::Duration::ZERO);
}
