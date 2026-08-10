use std::fs;
use std::path::Path;

use super::copy::copy_dir_recursive;

pub(super) fn sync_bridge_payload(src: &Path, dest: &Path, dir_name: &str) {
    fs::create_dir_all(dest).unwrap_or_else(|e| {
        panic!("mkdir {}: {e}", dest.display());
    });
    for name in ["package.json", "package-lock.json"] {
        let from = src.join(name);
        let to = dest.join(name);
        fs::copy(&from, &to).unwrap_or_else(|e| {
            panic!("copy {} -> {}: {e}", from.display(), to.display());
        });
    }
    let src_dist = src.join("dist");
    if src_dist.is_dir() {
        copy_dir_recursive(&src_dist, &dest.join("dist")).unwrap_or_else(|e| {
            panic!("copy dist for {dir_name}: {e}");
        });
        return;
    }
    copy_build_sources(src, dest, dir_name);
}

fn copy_build_sources(src: &Path, dest: &Path, dir_name: &str) {
    for name in ["tsconfig.json", "src"] {
        let from = src.join(name);
        let to = dest.join(name);
        if from.is_dir() {
            copy_dir_recursive(&from, &to).unwrap_or_else(|e| {
                panic!("copy {name} for {dir_name}: {e}");
            });
        } else if from.is_file() {
            fs::copy(&from, &to).unwrap_or_else(|e| {
                panic!("copy {} -> {}: {e}", from.display(), to.display());
            });
        }
    }
}
