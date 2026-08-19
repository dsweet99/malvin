mod copy;
mod npm;
mod sync;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use npm::{check_node_version, resolve_npm, run_npm};
use sync::sync_bridge_payload;

fn fnv1a64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;
    let mut hash = OFFSET;
    for byte in data {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

pub struct Bridge {
    pub dir_name: &'static str,
    pub package_marker: &'static str,
    pub label: &'static str,
    pub min_node: (u32, u32),
}

pub const BRIDGES: &[Bridge] = &[Bridge {
    dir_name: "cursor-sdk-bridge",
    package_marker: "@cursor/sdk/package.json",
    label: "Cursor SDK (@cursor/sdk)",
    min_node: (22, 13),
}];

pub fn run_build_script() {
    println!("cargo:rerun-if-env-changed=MALVIN_SKIP_SDK_BRIDGES");
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=HOME");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    emit_rerun_if_changed(&manifest_dir);

    if env::var_os("DOCS_RS").is_some() {
        return;
    }
    if env::var_os("MALVIN_SKIP_SDK_BRIDGES").is_some() {
        println!(
            "cargo:warning=MALVIN_SKIP_SDK_BRIDGES is set; Cursor SDK bridge was not installed"
        );
        return;
    }

    for bridge in BRIDGES {
        ensure_bridge(&manifest_dir, bridge);
    }
}

fn emit_rerun_if_changed(manifest_dir: &Path) {
    for bridge in BRIDGES {
        let dir = manifest_dir.join(bridge.dir_name);
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("package.json").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("package-lock.json").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("tsconfig.json").display()
        );
        if let Ok(entries) = fs::read_dir(dir.join("src")) {
            for entry in entries.flatten() {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            }
        }
    }
}

pub fn ensure_bridge(manifest_dir: &Path, bridge: &Bridge) {
    let src = manifest_dir.join(bridge.dir_name);
    assert!(
        src.join("package.json").is_file(),
        "missing {}; the malvin crate requires the {} bridge sources",
        src.join("package.json").display(),
        bridge.label
    );

    if in_tree_bridge_ready(&src, bridge) {
        return;
    }

    let dest = sdk_share_dir().join(bridge.dir_name);
    sync_bridge_payload(&src, &dest, bridge.dir_name);
    if share_bridge_ready(&src, &dest, bridge) {
        return;
    }

    install_npm_deps(&dest, bridge);
}

fn in_tree_bridge_ready(src: &Path, bridge: &Bridge) -> bool {
    src.join("node_modules")
        .join(bridge.package_marker)
        .is_file()
        && src.join("dist").join("bridge.js").is_file()
}

fn share_bridge_ready(src: &Path, dest: &Path, bridge: &Bridge) -> bool {
    let marker = dest.join("node_modules").join(bridge.package_marker);
    let dist_js = dest.join("dist").join("bridge.js");
    let stamp_path = dest.join(".malvin-npm-stamp");
    let lock_bytes = fs::read(src.join("package-lock.json")).unwrap_or_default();
    let lock_stamp = format!("{:x}", fnv1a64(&lock_bytes));
    marker.is_file()
        && dist_js.is_file()
        && fs::read_to_string(&stamp_path).unwrap_or_default().trim() == lock_stamp
}

fn install_npm_deps(dest: &Path, bridge: &Bridge) {
    let npm = resolve_npm();
    check_node_version(bridge);
    eprintln!(
        "malvin: installing {} into {}…",
        bridge.label,
        dest.display()
    );
    let dist_js = dest.join("dist").join("bridge.js");
    if dist_js.is_file() {
        run_npm(&npm, dest, &["ci", "--omit=dev"]);
    } else {
        run_npm(&npm, dest, &["ci"]);
        eprintln!(
            "malvin: building {} bridge (npm run build)…",
            bridge.dir_name
        );
        run_npm(&npm, dest, &["run", "build"]);
    }
    verify_install(dest, bridge);
    write_stamp(dest);
}

fn verify_install(dest: &Path, bridge: &Bridge) {
    let marker = dest.join("node_modules").join(bridge.package_marker);
    assert!(
        marker.is_file(),
        "after npm ci, {} is missing under {}. \
         Install Node >= {}.{}, ensure network access for npm, then retry.",
        bridge.package_marker,
        dest.display(),
        bridge.min_node.0,
        bridge.min_node.1
    );
    let dist_js = dest.join("dist").join("bridge.js");
    assert!(
        dist_js.is_file(),
        "after npm install, {} is missing; bridge TypeScript build failed",
        dist_js.display()
    );
}

fn write_stamp(dest: &Path) {
    let lock = dest.join("package-lock.json");
    let lock_bytes = fs::read(&lock).unwrap_or_default();
    let stamp_path = dest.join(".malvin-npm-stamp");
    fs::write(&stamp_path, format!("{:x}\n", fnv1a64(&lock_bytes))).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", stamp_path.display());
    });
}

pub fn sdk_share_dir() -> PathBuf {
    let home = env::var_os("HOME").filter(|v| !v.is_empty()).unwrap_or_else(|| {
        panic!(
            "malvin requires $HOME to install Cursor/Pi bridges under ~/.malvin_home/sdk-bridges/"
        )
    });
    PathBuf::from(home).join(".malvin_home").join("sdk-bridges")
}

#[cfg(test)]
mod tests;
