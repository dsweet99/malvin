use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

struct PiPatchSpec {
    patch_rel: &'static str,
    marker_rel: &'static str,
    marker: &'static str,
    cfg_name: &'static str,
    label: &'static str,
}

pub fn apply_pi_openrouter_cost_patch(manifest_dir: &Path) {
    const SPECS: &[PiPatchSpec] = &[
        PiPatchSpec {
            patch_rel: "admin/patches/pi_agent_rust-0.1.23-openrouter-cost.patch",
            marker_rel: "src/providers/openai.rs",
            marker: "take_openrouter_generation_ids",
            cfg_name: "malvin_pi_openrouter_patch",
            label: "OpenRouter billed cost",
        },
        PiPatchSpec {
            patch_rel: "admin/patches/pi_agent_rust-0.1.23-text-tool-recovery.patch",
            marker_rel: "src/agent.rs",
            marker: "recover_text_json_tool_calls",
            cfg_name: "malvin_pi_text_tool_recovery_patch",
            label: "local LLM text tool-call recovery",
        },
        PiPatchSpec {
            patch_rel: "admin/patches/pi_agent_rust-0.1.23-edit-arg-aliases.patch",
            marker_rel: "src/tools.rs",
            marker: "alias = \"old\", alias = \"old_text\"",
            cfg_name: "malvin_pi_edit_arg_aliases_patch",
            label: "edit/write arg aliases for local LLMs",
        },
    ];
    for spec in SPECS {
        apply_pi_patch(manifest_dir, spec);
    }
}

fn apply_pi_patch(manifest_dir: &Path, spec: &PiPatchSpec) {
    let patch = manifest_dir.join(spec.patch_rel);
    println!("cargo:rerun-if-changed={}", patch.display());

    if env::var_os("DOCS_RS").is_some() || env::var_os("MALVIN_SKIP_PI_PATCH").is_some() {
        return;
    }

    let Some(pi_src) = find_pi_agent_rust_0_1_23_src() else {
        println!(
            "cargo:warning=pi_agent_rust-0.1.23 source not found; {} patch skipped",
            spec.label
        );
        return;
    };

    let marker_path = pi_src.join(spec.marker_rel);
    if file_contains_marker(&marker_path, spec.marker) {
        println!("cargo:rustc-cfg={}", spec.cfg_name);
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
            println!("cargo:rustc-cfg={}", spec.cfg_name);
        }
        Ok(s) => {
            println!(
                "cargo:warning=pi {} patch exited with status {s} (may already be applied)",
                spec.label
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=failed to run patch for pi_agent_rust ({}): {e}",
                spec.label
            );
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

fn file_contains_marker(path: &Path, marker: &str) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .is_some_and(|body| body.contains(marker))
}
