use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn apply_pi_openrouter_cost_patch(manifest_dir: &Path) {
    let patch = manifest_dir.join("admin/patches/pi_agent_rust-0.1.23-openrouter-cost.patch");
    println!("cargo:rerun-if-changed={}", patch.display());

    if env::var_os("DOCS_RS").is_some() || env::var_os("MALVIN_SKIP_PI_PATCH").is_some() {
        return;
    }

    let Some(pi_src) = find_pi_agent_rust_0_1_23_src() else {
        println!(
            "cargo:warning=pi_agent_rust-0.1.23 source not found; OpenRouter billed cost patch skipped"
        );
        return;
    };

    let openai_rs = pi_src.join("src/providers/openai.rs");
    if openrouter_patch_already_applied(&openai_rs) {
        println!("cargo:rustc-cfg=malvin_pi_openrouter_patch");
        return;
    }

    let status = Command::new("patch")
        .arg("-p1")
        .arg("-N")
        .arg("-i")
        .arg(&patch)
        .current_dir(&pi_src)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("cargo:rustc-cfg=malvin_pi_openrouter_patch");
        }
        Ok(s) => {
            println!(
                "cargo:warning=pi OpenRouter cost patch exited with status {s} (may already be applied)"
            );
        }
        Err(e) => {
            println!("cargo:warning=failed to run patch for pi_agent_rust: {e}");
        }
    }
}

fn find_pi_agent_rust_0_1_23_src() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let registry = PathBuf::from(home).join(".cargo/registry/src");
    let entries = std::fs::read_dir(&registry).ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("pi_agent_rust-0.1.23");
        if candidate.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
    }
    None
}

fn openrouter_patch_already_applied(openai_rs: &Path) -> bool {
    std::fs::read_to_string(openai_rs)
        .ok()
        .is_some_and(|body| body.contains("take_openrouter_generation_ids"))
}
