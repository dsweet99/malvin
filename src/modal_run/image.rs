use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::config::ModalConfig;
use crate::npm_bridge_install::fnv1a64;

pub const DEFAULT_BASE: &str = "node:22-trixie-slim";
pub const TOOLCHAIN: &str = "RUN apt-get update && apt-get install -y --no-install-recommends \
     git ca-certificates curl python3 build-essential && rm -rf /var/lib/apt/lists/*";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageSpec {
    pub name: String,
    pub base: String,
    pub layers: Vec<Vec<String>>,
    pub binary: Option<PathBuf>,
    pub min_glibc: Option<String>,
}

#[must_use]
pub fn uploadable_binary() -> Option<PathBuf> {
    if cfg!(all(target_os = "linux", target_arch = "x86_64", target_env = "gnu")) {
        std::env::current_exe().ok()
    } else {
        None
    }
}

#[must_use]
pub fn crates_io_layer(version: &str) -> Vec<String> {
    vec![
        "RUN apt-get update && apt-get install -y --no-install-recommends libcap-ng-dev \
         && rm -rf /var/lib/apt/lists/*"
            .to_string(),
        format!(
            "RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal \
             && /root/.cargo/bin/cargo install malvin --version {version} --locked \
             && cp /root/.cargo/bin/malvin /usr/local/bin/malvin \
             && rm -rf /root/.cargo/registry /root/.rustup"
        ),
    ]
}

fn host_glibc() -> Option<String> {
    let out = std::process::Command::new("getconf")
        .arg("GNU_LIBC_VERSION")
        .output()
        .ok()?;
    super::backends::parse_version_token(&String::from_utf8_lossy(&out.stdout))
}

#[must_use]
pub fn image_name(base: &str, layers: &[Vec<String>], payload_hash: u64) -> String {
    let mut text = format!("{base}\n{VERSION}\n{payload_hash:016x}\n");
    for line in layers.iter().flatten() {
        text.push_str(line);
        text.push('\n');
    }
    format!("malvin-bin:{VERSION}-{:016x}", fnv1a64(text.as_bytes()))
}

fn binary_hash(path: &Path) -> Result<u64, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    Ok(fnv1a64(&bytes))
}

pub fn image_spec(cfg: &ModalConfig, model: &str, binary: Option<PathBuf>) -> Result<ImageSpec, String> {
    let base = cfg.image.clone().unwrap_or_else(|| DEFAULT_BASE.to_string());
    let mut layers = Vec::new();
    if cfg.image.is_none() {
        layers.push(vec![TOOLCHAIN.to_string()]);
    }
    layers.extend(super::backends::backend_layer(model, &crate::user_home_dir()));
    if !cfg.setup.is_empty() {
        layers.push(cfg.setup.clone());
    }
    let payload_hash = if let Some(path) = &binary {
        binary_hash(path)?
    } else {
        layers.push(crates_io_layer(VERSION));
        0
    };
    Ok(ImageSpec {
        name: image_name(&base, &layers, payload_hash),
        min_glibc: binary.as_ref().and_then(|_| host_glibc()),
        base,
        layers,
        binary,
    })
}

impl ImageSpec {
    #[must_use]
    pub fn request(&self, force: bool) -> Value {
        json!({
            "name": self.name,
            "base": self.base,
            "layers": self.layers,
            "binary": self.binary.as_ref().map(|p| p.display().to_string()),
            "min_glibc": self.min_glibc,
            "need_node": true,
            "force": force,
        })
    }
}
