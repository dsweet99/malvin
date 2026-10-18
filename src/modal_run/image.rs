use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use super::config::ModalConfig;
use crate::npm_bridge_install::fnv1a64;

pub const DEFAULT_BASE: &str = "node:22-trixie-slim";
pub const TOOLCHAIN: &str = "RUN apt-get update && apt-get install -y --no-install-recommends \
     git ca-certificates curl python3 build-essential && rm -rf /var/lib/apt/lists/*";
/// Node 22 LTS tarball used when the base image does not already include Node.
const NODE_TARBALL: &str = "https://nodejs.org/dist/v22.23.3/node-v22.23.3-linux-x64.tar.xz";
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// A registry image and the newest glibc it provides. Newer glibc runs older binaries.
struct GlibcBase {
    image: &'static str,
    major: u64,
    minor: u64,
}

/// `DEFAULT_BASE` is Debian 13 (glibc 2.41). Ubuntu images cover hosts whose glibc is newer
/// than that, because the official Node image has no glibc 2.42+ tag.
const GLIBC_BASES: &[GlibcBase] = &[
    GlibcBase {
        image: DEFAULT_BASE,
        major: 2,
        minor: 41,
    },
    GlibcBase {
        image: "ubuntu:26.04",
        major: 2,
        minor: 43,
    },
    GlibcBase {
        image: "ubuntu:26.10",
        major: 2,
        minor: 44,
    },
];

#[must_use]
pub fn base_for_host_glibc(host: Option<&str>) -> &'static str {
    let Some((major, minor)) = host.and_then(parse_glibc) else {
        return DEFAULT_BASE;
    };
    GLIBC_BASES
        .iter()
        .find(|base| (base.major, base.minor) >= (major, minor))
        .map(|base| base.image)
        .unwrap_or(GLIBC_BASES[GLIBC_BASES.len() - 1].image)
}

fn parse_glibc(text: &str) -> Option<(u64, u64)> {
    let mut parts = text.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

#[must_use]
pub fn toolchain_commands(base_includes_node: bool) -> Vec<String> {
    if base_includes_node {
        return vec![TOOLCHAIN.to_string()];
    }
    vec![
        "RUN apt-get update && apt-get install -y --no-install-recommends \
         git ca-certificates curl python3 build-essential xz-utils \
         && rm -rf /var/lib/apt/lists/*"
            .to_string(),
        format!("RUN curl -fsSL {NODE_TARBALL} | tar -xJ -C /usr/local --strip-components=1"),
    ]
}

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
    if cfg!(all(
        target_os = "linux",
        target_arch = "x86_64",
        target_env = "gnu"
    )) {
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

fn config_and_backend_layers(
    cfg: &ModalConfig,
    model: &str,
    base_includes_node: bool,
) -> Result<Vec<Vec<String>>, String> {
    let mut layers = Vec::new();
    if cfg.image.is_none() {
        layers.push(toolchain_commands(base_includes_node));
    }
    layers.extend(super::backend_setup::setup_for(model)?.image_layer(&crate::user_home_dir()));
    if !cfg.setup.is_empty() {
        layers.push(cfg.setup.clone());
    }
    Ok(layers)
}

pub fn image_spec(
    cfg: &ModalConfig,
    model: &str,
    binary: Option<PathBuf>,
) -> Result<ImageSpec, String> {
    let host = binary.as_ref().and_then(|_| host_glibc());
    let base = cfg.image.clone().unwrap_or_else(|| {
        if binary.is_some() {
            base_for_host_glibc(host.as_deref()).to_string()
        } else {
            DEFAULT_BASE.to_string()
        }
    });
    let mut layers = config_and_backend_layers(cfg, model, base == DEFAULT_BASE)?;
    let payload_hash = if let Some(path) = &binary {
        binary_hash(path)?
    } else {
        layers.push(crates_io_layer(VERSION));
        0
    };
    Ok(ImageSpec {
        name: image_name(&base, &layers, payload_hash),
        min_glibc: host,
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
