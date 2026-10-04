#[path = "src/advice_registry_build.rs"]
mod advice_registry_build;
#[path = "src/sdk_bridge_build/mod.rs"]
mod sdk_bridge_build;

fn main() {
    advice_registry_build::generate();
    sdk_bridge_build::run_build_script();
}
