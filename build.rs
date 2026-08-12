//! Cargo build script: install Cursor / Cursor SDK npm bridges.

#[path = "src/sdk_bridge_build/mod.rs"]
mod sdk_bridge_build;

fn main() {
    sdk_bridge_build::run_build_script();
}
