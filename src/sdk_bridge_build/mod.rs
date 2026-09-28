mod lld;
mod pi_patch;

use std::env;
use std::path::PathBuf;

use lld::{emit_dev_dynamic_rpaths, emit_fast_bin_linker_args};

pub fn run_build_script() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=HOME");
    println!("cargo:rerun-if-env-changed=MALVIN_DISABLE_LLD");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    pi_patch::apply_pi_openrouter_cost_patch(&manifest_dir);
    emit_fast_bin_linker_args();
    emit_dev_dynamic_rpaths();
}

#[cfg(test)]
mod tests;
