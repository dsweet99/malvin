use std::collections::HashMap;

use super::{
    list_pi_provider_auth_sync, parse_list_providers_table, provider_authenticated_from_map,
};

const SAMPLE: &str = "\
provider                name                       aliases                                                                auth env                                                                                                              api                    
----------------------  -------------------------  ---------------------------------------------------------------------  --------------------------------------------------------------------------------------------------------------------  -----------------------
openai                  OpenAI                                                                                            OPENAI_API_KEY                                                                                                        openai-responses       
openrouter              OpenRouter                 open-router                                                            OPENROUTER_API_KEY                                                                                                    openai-completions     
302ai                   302.AI                                                                                            302AI_API_KEY                                                                                                         openai-completions     
deepseek                DeepSeek                   deep-seek                                                              DEEPSEEK_API_KEY                                                                                                      openai-completions     
google                  Google Gemini              gemini                                                                 GOOGLE_API_KEY, GEMINI_API_KEY                                                                                        google-generative-ai   
llamacpp                llama.cpp                  llama-cpp, llama.cpp, llama-server                                                                                                                                                           openai-completions     
";

#[test]
fn parse_list_providers_reads_keys_and_aliases() {
    let map = parse_list_providers_table(SAMPLE);
    assert_eq!(map.get("openai").map(Vec::as_slice), Some(&["OPENAI_API_KEY".to_string()][..]));
    assert_eq!(
        map.get("302ai").map(Vec::as_slice),
        Some(&["302AI_API_KEY".to_string()][..])
    );
    assert_eq!(
        map.get("openrouter").map(Vec::as_slice),
        Some(&["OPENROUTER_API_KEY".to_string()][..])
    );
    assert_eq!(
        map.get("open-router").map(Vec::as_slice),
        Some(&["OPENROUTER_API_KEY".to_string()][..])
    );
    assert_eq!(
        map.get("deep-seek").map(Vec::as_slice),
        Some(&["DEEPSEEK_API_KEY".to_string()][..])
    );
    assert_eq!(
        map.get("google").unwrap(),
        &["GOOGLE_API_KEY".to_string(), "GEMINI_API_KEY".to_string()]
    );
    assert_eq!(map.get("llamacpp").map(Vec::as_slice), Some(&[][..]));
    assert!(map.get("llama-server").is_some_and(Vec::is_empty));
}

#[test]
fn parse_list_providers_does_not_panic_on_non_ascii_row_text() {
    let header = "provider                name                       aliases                                                                auth env                                                                                                              api\n----------------------  -------------------------  ---------------------------------------------------------------------  --------------------------------------------------------------------------------------------------------------------  -----------------------\n";
    let row = format!(
        "{}é{}OpenAI{}OPENAI_API_KEY{}openai-responses\n",
        "a".repeat(23),
        " ".repeat(25),
        " ".repeat(70),
        " ".repeat(100),
    );
    let map = parse_list_providers_table(&format!("{header}{row}"));
    assert!(map.is_empty());
}

#[test]
fn parse_list_providers_skips_count_footer() {
    let text = format!("{SAMPLE}94 providers available.\n");
    let map = parse_list_providers_table(&text);
    assert!(!map.contains_key("94 providers available."));
    assert!(map.contains_key("openai"));
}

#[test]
fn provider_authenticated_from_map_honors_env_and_unknown() {
    let mut map = HashMap::new();
    map.insert("openai".into(), vec!["OPENAI_API_KEY".into()]);
    map.insert("llamacpp".into(), vec![]);
    crate::acp::with_env("OPENAI_API_KEY", None, || {
        assert!(!provider_authenticated_from_map("openai", &map));
        assert!(provider_authenticated_from_map("llamacpp", &map));
        assert!(provider_authenticated_from_map("unknown", &map));
    });
    crate::acp::with_env("OPENAI_API_KEY", Some("k"), || {
        assert!(provider_authenticated_from_map("openai", &map));
    });
}

#[cfg(unix)]
#[test]
fn list_pi_provider_auth_sync_parses_fake_pi() {
    use std::os::unix::fs::PermissionsExt;

    use crate::test_utils::test_env_lock;

    let _lock = test_env_lock();
    let dir = tempfile::tempdir().expect("tmpdir");
    let path = dir.path().join("ok-pi");
    std::fs::write(
        &path,
        format!("#!/bin/sh\ncat <<'EOF'\n{SAMPLE}EOF\n"),
    )
    .expect("write");
    let mut perms = std::fs::metadata(&path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).expect("chmod");
    let prior = std::env::var_os("MALVIN_PI");
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_PI", path.as_os_str());
    }
    let map = list_pi_provider_auth_sync().expect("fake providers");
    #[allow(unsafe_code)]
    unsafe {
        match prior {
            Some(v) => std::env::set_var("MALVIN_PI", v),
            None => std::env::remove_var("MALVIN_PI"),
        }
    }
    assert!(map.contains_key("openai"));
    assert!(map.contains_key("open-router"));
}
