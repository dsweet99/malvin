#[path = "src/cgroup_build.rs"]
mod cgroup_build;

fn main() {
    cgroup_build::run_build_script_from_cargo_env();
}
