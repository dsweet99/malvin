use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn emit_fast_bin_linker_args() {
    if env::var_os("MALVIN_DISABLE_LLD").is_some() {
        return;
    }
    if emit_mold_bin_linker_args() {
        return;
    }
    emit_rustup_lld_bin_linker_args();
}

pub(super) fn emit_dev_dynamic_rpaths() {
    if env::var_os("MALVIN_DISABLE_DYNAMIC_RPATH").is_some() {
        return;
    }
    println!("cargo:rerun-if-env-changed=MALVIN_DISABLE_DYNAMIC_RPATH");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/deps");
    let Some(libdir) = rustc_target_libdir() else {
        return;
    };
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", libdir.display());
    stage_libstd_for_rpath(&libdir);
}

fn stage_libstd_for_rpath(libdir: &Path) {
    let Ok(entries) = fs::read_dir(libdir) else {
        return;
    };
    let Some(deps_dir) = cargo_deps_dir_from_out_dir() else {
        return;
    };
    let _ = fs::create_dir_all(&deps_dir);
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !is_libstd_shared_object(name) {
            continue;
        }
        copy_libstd_so_if_missing(&entry.path(), &deps_dir.join(name));
    }
}

fn cargo_deps_dir_from_out_dir() -> Option<PathBuf> {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR")?);
    out_dir
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("deps"))
}

fn is_libstd_shared_object(name: &str) -> bool {
    name.starts_with("libstd-")
        && Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("so"))
}

fn copy_libstd_so_if_missing(src: &Path, dest: &Path) {
    if dest.exists() {
        return;
    }
    if let Err(e) = fs::copy(src, dest) {
        eprintln!(
            "cargo:warning=malvin: stage {} for rpath: {e}",
            src.display()
        );
    }
}

fn rustc_target_libdir() -> Option<PathBuf> {
    let rustc = env::var_os("RUSTC").map_or_else(|| PathBuf::from("rustc"), PathBuf::from);
    let output = Command::new(rustc)
        .args(["--print", "target-libdir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let libdir = String::from_utf8_lossy(&output.stdout);
    let libdir = PathBuf::from(libdir.trim());
    libdir.is_dir().then_some(libdir)
}

fn emit_mold_bin_linker_args() -> bool {
    let Some(mold) = resolve_mold_bin() else {
        return false;
    };
    let Some(shim_dir) = mold_shim_dir() else {
        return false;
    };
    if let Err(e) = fs::create_dir_all(&shim_dir) {
        eprintln!(
            "cargo:warning=malvin: mold shim dir {}: {e}",
            shim_dir.display()
        );
        return false;
    }
    let ld_shim = shim_dir.join("ld");
    if !ld_shim.exists() {
        #[cfg(unix)]
        {
            if let Err(e) = std::os::unix::fs::symlink(&mold, &ld_shim) {
                eprintln!(
                    "cargo:warning=malvin: mold shim symlink {}: {e}",
                    ld_shim.display()
                );
                return false;
            }
        }
        #[cfg(not(unix))]
        {
            let _ = mold;
            return false;
        }
    }
    println!("cargo:rustc-link-arg-bins=-B{}", shim_dir.display());
    true
}

fn emit_rustup_lld_bin_linker_args() {
    let Some(gcc_ld) = rustc_gcc_ld_dir() else {
        return;
    };
    if !gcc_ld.join("ld.lld").is_file() {
        return;
    }
    println!("cargo:rustc-link-arg-bins=-B{}", gcc_ld.display());
    println!("cargo:rustc-link-arg-bins=-fuse-ld=lld");
}

fn resolve_mold_bin() -> Option<PathBuf> {
    if let Some(path) = env::var_os("MOLD") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    let output = Command::new("sh")
        .args(["-c", "command -v mold"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout);
    let path = PathBuf::from(path.trim());
    path.is_file().then_some(path)
}

fn mold_shim_dir() -> Option<PathBuf> {
    let out = env::var_os("OUT_DIR")?;
    Some(PathBuf::from(out).join("mold-bin-shim"))
}

pub(super) fn rustc_gcc_ld_dir() -> Option<PathBuf> {
    let sysroot = rustc_sysroot()?;
    let target = cargo_target_triple();
    Some(
        PathBuf::from(sysroot)
            .join("lib/rustlib")
            .join(target)
            .join("bin/gcc-ld"),
    )
}

fn rustc_sysroot() -> Option<String> {
    let rustc = env::var_os("RUSTC").map_or_else(|| PathBuf::from("rustc"), PathBuf::from);
    let output = Command::new(rustc)
        .args(["--print", "sysroot"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sysroot = String::from_utf8_lossy(&output.stdout);
    let sysroot = sysroot.trim();
    if sysroot.is_empty() {
        None
    } else {
        Some(sysroot.to_string())
    }
}

fn cargo_target_triple() -> String {
    env::var("TARGET")
        .or_else(|_| env::var("HOST"))
        .unwrap_or_else(|_| "x86_64-unknown-linux-gnu".to_string())
}
