use std::path::Path;

pub const PI_DIR: &str = "/opt/malvin-pi";
pub const PI_ENTRY: &str =
    "/opt/malvin-pi/node_modules/@earendil-works/pi-coding-agent/dist/bundle/rpc-entry.js";
pub(super) const PI_PACKAGE: &str = "@earendil-works/pi-coding-agent";

#[must_use]
pub fn parse_version_token(text: &str) -> Option<String> {
    let token = text.split_whitespace().last()?.trim_start_matches('v');
    let valid = !token.is_empty()
        && token
            .split('.')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
    valid.then(|| token.to_string())
}

pub(super) fn local_codex_version() -> String {
    std::process::Command::new("codex")
        .arg("--version")
        .output()
        .ok()
        .and_then(|o| parse_version_token(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_else(|| "latest".to_string())
}

#[must_use]
pub fn local_pi_version(home: &Path) -> String {
    let manifest = home
        .join(crate::MALVIN_USER_HOME_DIR)
        .join("sdk-bridges/node_modules")
        .join(PI_PACKAGE)
        .join("package.json");
    std::fs::read_to_string(manifest)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v["version"].as_str().and_then(parse_version_token))
        .unwrap_or_else(|| "latest".to_string())
}
