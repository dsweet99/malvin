#[path = "src/sdk_bridge_build/mod.rs"]
mod sdk_bridge_build;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(malvin_pi_openrouter_patch)");
    sdk_bridge_build::run_build_script();
}
