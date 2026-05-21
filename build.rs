#[path = "src/cgroup_build.rs"]
mod cgroup_build;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(malvin_have_writable_cgroups)");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux") {
        return;
    }
    if cgroup_build::probe_writable_cgroup_parent().is_some() {
        println!("cargo:rustc-cfg=malvin_have_writable_cgroups");
    }
}
