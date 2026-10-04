mod lld;

use lld::{emit_dev_dynamic_rpaths, emit_fast_bin_linker_args};

pub fn run_build_script() {
    println!("cargo:rerun-if-env-changed=DOCS_RS");
    println!("cargo:rerun-if-env-changed=HOME");
    println!("cargo:rerun-if-env-changed=MALVIN_DISABLE_LLD");

    emit_fast_bin_linker_args();
    emit_dev_dynamic_rpaths();
}

#[cfg(test)]
mod tests;
