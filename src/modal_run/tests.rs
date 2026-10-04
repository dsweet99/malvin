use serde_json::json;

use super::config::{ModalConfig, parse_modal_config, read_config_root, remote_config_text};
use super::credentials::{FORWARDED_ENV, file_logins, forwarded_env, preflight};
use super::image::{DEFAULT_BASE, TOOLCHAIN, crates_io_layer, image_name, image_spec};

fn cfg(text: &str, mem: u64) -> Result<ModalConfig, String> {
    parse_modal_config(&text.parse::<toml::Value>().unwrap(), mem)
}

#[test]
fn modal_config_defaults_follow_the_memory_cap() {
    let c = cfg("mem_limit_gb = 16", 16).unwrap();
    assert_eq!((c.cpu, c.memory_gb, c.timeout_h), (2, 18, 24));
    assert_eq!(c.remote_mem_limit_gb, 16);
    assert!(c.image.is_none() && c.setup.is_empty());
}

#[test]
fn modal_config_reads_overrides_and_shrinks_the_remote_cap() {
    let text = "[modal]\ncpu = 4\nmemory_gb = 8\ntimeout_h = 3\nimage = \"python:3.12\"\nsetup = [\"RUN pip install x\"]";
    let c = cfg(text, 16).unwrap();
    assert_eq!((c.cpu, c.memory_gb, c.timeout_h), (4, 8, 3));
    assert_eq!(c.image.as_deref(), Some("python:3.12"));
    assert_eq!(c.setup, vec!["RUN pip install x".to_string()]);
    assert_eq!(c.remote_mem_limit_gb, 6);
}

#[test]
fn modal_config_rejects_bad_values() {
    assert!(cfg("[modal]\ntimeout_h = 25", 4).unwrap_err().contains("24 hours"));
    assert!(cfg("[modal]\ncpu = 0", 4).unwrap_err().contains("positive"));
    assert!(cfg("[modal]\nsetup = \"RUN x\"", 4).unwrap_err().contains("list"));
    assert!(cfg("[modal]\nsetup = [1]", 4).unwrap_err().contains("strings"));
}

#[test]
fn remote_config_keeps_settings_and_sets_the_cap() {
    let root: toml::Value = "theme = \"dark\"\nmem_limit_gb = 16\n[modal]\nmemory_gb = 8".parse().unwrap();
    let c = parse_modal_config(&root, 16).unwrap();
    let text = remote_config_text(&root, &c).unwrap();
    let back: toml::Value = text.parse().unwrap();
    assert_eq!(back["mem_limit_gb"].as_integer(), Some(6));
    assert_eq!(back["theme"].as_str(), Some("dark"));
}

#[test]
fn read_config_root_treats_a_missing_file_as_empty() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(read_config_root(&tmp.path().join("nope.toml")).unwrap().as_table().unwrap().is_empty());
    std::fs::write(tmp.path().join("bad.toml"), "= x").unwrap();
    assert!(read_config_root(&tmp.path().join("bad.toml")).is_err());
}

#[test]
fn image_name_changes_with_every_input() {
    let layers = vec![vec!["RUN a".to_string()]];
    let base = image_name("b", &layers, 1);
    assert!(base.starts_with(&format!("malvin-bin:{}-", env!("CARGO_PKG_VERSION"))));
    assert_eq!(base, image_name("b", &layers, 1));
    assert_ne!(base, image_name("c", &layers, 1));
    assert_ne!(base, image_name("b", &[vec!["RUN b".to_string()]], 1));
    assert_ne!(base, image_name("b", &layers, 2));
}

#[test]
fn image_spec_layers_toolchain_setup_and_source() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("malvin");
    std::fs::write(&bin, b"binary").unwrap();
    let mut c = cfg("", 4).unwrap();
    c.setup = vec!["RUN extra".to_string()];
    let spec = image_spec(&c, "cursor:auto", Some(bin.clone())).unwrap();
    assert_eq!(spec.base, DEFAULT_BASE);
    assert_eq!(spec.layers, vec![vec![TOOLCHAIN.to_string()], c.setup.clone()]);
    assert_eq!(spec.binary, Some(bin));
    c.image = Some("custom:1".to_string());
    let custom = image_spec(&c, "codex:gpt", None).unwrap();
    assert_eq!(custom.layers.len(), 3);
    assert!(custom.layers[0][0].contains("@openai/codex@"));
    assert_eq!(custom.layers[1..], [c.setup.clone(), crates_io_layer(env!("CARGO_PKG_VERSION"))]);
    assert!(custom.min_glibc.is_none());
    assert_eq!(custom.request(true)["force"], json!(true));
    assert!(crates_io_layer("9.9.9")[1].contains("--version 9.9.9 --locked"));
}

#[test]
fn forwarded_env_keeps_only_set_provider_keys() {
    let env = forwarded_env(|k| match k {
        "OPENAI_API_KEY" => Some("sk".to_string()),
        "CURSOR_API_KEY" => Some("  ".to_string()),
        "PATH" => Some("/bin".to_string()),
        _ => None,
    });
    assert_eq!(env.keys().collect::<Vec<_>>(), vec!["OPENAI_API_KEY"]);
    assert!(FORWARDED_ENV.contains(&"OPENROUTER_API_KEY"));
}

#[test]
fn file_logins_depend_on_the_backend() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path();
    std::fs::create_dir_all(home.join(".codex")).unwrap();
    std::fs::create_dir_all(home.join(".pi/agent")).unwrap();
    std::fs::write(home.join(".codex/auth.json"), "{}").unwrap();
    std::fs::write(home.join(".pi/agent/auth.json"), "{}").unwrap();
    crate::acp::with_env("PI_CODING_AGENT_DIR", None, || {
        assert_eq!(file_logins("codex:x", home), vec![home.join(".codex/auth.json")]);
        assert_eq!(file_logins("pi:openai/x", home), vec![home.join(".pi/agent/auth.json")]);
        assert!(file_logins("cursor:auto", home).is_empty());
    });
}

#[test]
fn preflight_names_missing_modal_credentials() {
    let _g = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    crate::acp::with_env("MODAL_TOKEN_ID", None, || {
        let err = preflight("pi:openai/x", tmp.path()).unwrap_err();
        assert!(err.contains("modal setup") && err.contains("MODAL_TOKEN_ID"), "{err}");
        std::fs::write(tmp.path().join(".modal.toml"), "").unwrap();
        assert!(preflight("pi:openai/x", tmp.path()).is_ok());
        crate::acp::with_env("CURSOR_API_KEY", None, || {
            crate::acp::with_env("CURSOR_AGENT_API_KEY", None, || {
                crate::acp::with_env("AGENT_API_KEY", None, || {
                    assert!(preflight("cursor:auto", tmp.path()).unwrap_err().contains("CURSOR_API_KEY"));
                });
            });
        });
    });
}
