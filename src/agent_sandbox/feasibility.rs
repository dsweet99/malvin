use std::io::Read;
use std::path::{Path, PathBuf};

pub fn agent_install_root(agent_bin: &Path) -> PathBuf {
    agent_bin
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

pub fn linux_node_in_bundle(agent_bin: &Path) -> Option<PathBuf> {
    let node = agent_install_root(agent_bin).join("node");
    if node.is_file() && path_is_elf_executable(&node) {
        Some(node)
    } else {
        None
    }
}

fn path_is_elf_executable(path: &Path) -> bool {
    let Ok(mut head) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0_u8; 4];
    if head.read_exact(&mut buf).is_err() {
        return false;
    }
    buf == [0x7f, b'E', b'L', b'F']
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elf_magic_detected() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("fake-node");
        std::fs::write(&path, [0x7f, b'E', b'L', b'F', 0]).expect("write");
        assert!(path_is_elf_executable(&path));
    }
}
