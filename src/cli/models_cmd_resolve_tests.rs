use super::{resolved_model_json, write_resolved_model};
use malvin::model_id::parse_model_id;
use malvin::test_utils::with_isolated_home;
use malvin::workspace_paths::malvin_config_path;

fn json_for(raw: &str) -> serde_json::Value {
    let model = parse_model_id(raw).expect("valid model id");
    serde_json::from_str(&resolved_model_json(&model)).expect("json")
}

#[test]
fn resolved_json_reports_backend_provider_and_locality() {
    let codex = json_for("  codex:gpt-5  ");
    assert_eq!(codex["canonical"], "codex:gpt-5");
    assert_eq!(codex["backend"], "codex");
    assert!(codex["provider"].is_null());
    assert_eq!(codex["local"], false);

    let ollama = json_for("rpi:ollama/qwen");
    assert_eq!(ollama["backend"], "rpi");
    assert_eq!(ollama["provider"], "ollama");
    assert_eq!(ollama["local"], true);

    let bare_local = json_for("rpi:local/qwen");
    assert_eq!(bare_local["canonical"], "rpi:local/qwen");
    assert_eq!(bare_local["provider"], "ollama");
    assert_eq!(bare_local["local"], true);

    let llamacpp = json_for("rpi:local/llamacpp/m");
    assert_eq!(llamacpp["provider"], "llamacpp");
    assert_eq!(llamacpp["local"], true);

    let cloud = json_for("pi:openrouter/x-ai/grok");
    assert_eq!(cloud["backend"], "pi");
    assert_eq!(cloud["provider"], "openrouter");
    assert_eq!(cloud["local"], false);
}

#[test]
fn write_resolved_model_expands_nicknames_and_rejects_bad_ids() {
    with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(
            &path,
            "[nicknames]\nastra = \"codex:gpt-5\"\n[agent]\nmodel = \"cursor:auto\"\n",
        )
        .expect("write");
        let mut out = Vec::new();
        write_resolved_model("astra", &mut out).expect("resolve nickname");
        let text = String::from_utf8(out).expect("utf8");
        assert!(text.ends_with('\n'));
        let value: serde_json::Value = serde_json::from_str(text.trim()).expect("json");
        assert_eq!(value["canonical"], "codex:gpt-5");
        assert_eq!(value["backend"], "codex");

        let mut sink = Vec::new();
        assert!(write_resolved_model("RPI:ollama/x", &mut sink).is_err());
        assert!(sink.is_empty());
    });
}
