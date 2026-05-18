use std::fs;

pub fn read_memory_events_oom_kill_count(text: &str) -> u64 {
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else {
            continue;
        };
        if key == "oom_kill" || key == "oom" {
            if let Some(value) = parts.next() {
                if let Ok(n) = value.parse::<u64>() {
                    return n;
                }
            }
        }
    }
    0
}

pub fn v1_under_oom(cgroup_dir: &Path) -> bool {
    let oom_control = cgroup_dir.join("memory.oom_control");
    let Ok(text) = fs::read_to_string(oom_control) else {
        return false;
    };
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let (Some(key), Some(val)) = (parts.next(), parts.next()) else {
            continue;
        };
        if key == "under_oom" && val == "1" {
            return true;
        }
    }
    false
}

pub fn cgroup_v2_mount() -> Option<PathBuf> {
    const ROOT: &str = "/sys/fs/cgroup";
    if Path::new(ROOT).join("cgroup.controllers").is_file() {
        return Some(PathBuf::from(ROOT));
    }
    let unified = Path::new(ROOT).join("unified");
    if unified.join("cgroup.controllers").is_file() {
        return Some(unified);
    }
    None
}

pub fn self_cgroup_v2_relative_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("0::").map(str::to_string))
}

pub fn cgroup_memory_max_is_limited(cgroup_dir: &Path, expected_bytes: u64) -> bool {
    if let Some(max) = read_v2_memory_max(cgroup_dir) {
        return max > 0 && max <= expected_bytes;
    }
    read_v1_memory_limit(cgroup_dir).is_some_and(|max| max > 0 && max <= expected_bytes)
}

pub fn parse_memory_limit_bytes(text: &str) -> Option<u64> {
    if text.eq_ignore_ascii_case("max") {
        return None;
    }
    text.parse::<u64>().ok()
}

#[cfg(test)]
pub fn parse_memory_events_oom(text: &str) -> bool {
    read_memory_events_oom_kill_count(text) > 0
}

pub fn resolve_writable_cgroup_parent() -> Option<PathBuf> {
    if let Some(parent) = resolve_cgroup_v2_parent() {
        return Some(parent);
    }
    resolve_cgroup_v1_memory_parent()
}

pub fn write_memory_limit(cgroup_dir: &Path, bytes: u64) -> bool {
    if write_v2_memory_max(cgroup_dir, bytes) {
        return true;
    }
    write_v1_memory_limit(cgroup_dir, bytes)
}

pub fn resolve_cgroup_v2_parent() -> Option<PathBuf> {
    let mount = cgroup_v2_mount()?;
    let rel = self_cgroup_v2_relative_path()?;
    let parent = mount.join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("cgroup.procs").is_file())
}

pub fn resolve_cgroup_v1_memory_parent() -> Option<PathBuf> {
    let rel = self_cgroup_v1_memory_relative_path()?;
    let parent = PathBuf::from("/sys/fs/cgroup/memory").join(rel.trim_start_matches('/'));
    probe_writable_parent(&parent, |dir| dir.join("tasks").is_file())
}

pub fn probe_writable_parent(start: &Path, marker: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if marker(&current) {
            let probe = current.join(format!(".malvin-probe-{}", std::process::id()));
            if fs::create_dir(&probe).is_ok() {
                let _ = fs::remove_dir(&probe);
                return Some(current);
            }
        }
        if !current.pop() {
            return None;
        }
    }
}

fn write_v2_memory_max(cgroup_dir: &Path, bytes: u64) -> bool {
    let path = cgroup_dir.join("memory.max");
    if !path.parent().is_some_and(std::path::Path::is_dir) {
        return false;
    }
    fs::write(path, bytes.to_string()).is_ok()
}

fn write_v1_memory_limit(cgroup_dir: &Path, bytes: u64) -> bool {
    let path = cgroup_dir.join("memory.limit_in_bytes");
    fs::write(path, bytes.to_string()).is_ok()
}

pub fn read_v2_memory_max(cgroup_dir: &Path) -> Option<u64> {
    let text = fs::read_to_string(cgroup_dir.join("memory.max")).ok()?;
    parse_memory_limit_bytes(text.trim())
}

pub fn read_v1_memory_limit(cgroup_dir: &Path) -> Option<u64> {
    let text = fs::read_to_string(cgroup_dir.join("memory.limit_in_bytes")).ok()?;
    parse_memory_limit_bytes(text.trim())
}

pub fn self_cgroup_v1_memory_relative_path() -> Option<String> {
    let text = fs::read_to_string("/proc/self/cgroup").ok()?;
    text.lines().find_map(|line| {
        let (_, rest) = line.split_once("memory:")?;
        Some(rest.to_string())
    })
}
