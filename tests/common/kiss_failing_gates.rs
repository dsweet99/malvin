use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub fn write_failing_command(path: &Path, trace: &Path) {
    let name = path.file_name().unwrap().to_string_lossy();
    std::fs::write(
        path,
        format!(
            "#!/usr/bin/env sh\necho \"{name} $@\" >> \"{}\"\nexit 1\n",
            trace.display()
        ),
    )
    .expect("write failing command");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

pub fn write_failing_gate_tools(bin_dir: &Path, trace: &Path) {
    for name in ["kiss", "cargo", "ruff", "pytest"] {
        write_failing_command(&bin_dir.join(name), trace);
    }
}
