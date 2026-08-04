use super::{ForkState, workspace_manifest_hash};

#[test]
fn workspace_manifest_hash_is_stable_for_same_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let a = workspace_manifest_hash(tmp.path());
    let b = workspace_manifest_hash(tmp.path());
    assert_eq!(a, b);
    assert!(!a.is_empty());
}

#[test]
fn capture_records_memory_and_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), "h1", "prev");
    assert_eq!(checkpoint.history, "h1");
    assert_eq!(checkpoint.previous_response, "prev");
    assert_eq!(
        checkpoint.workspace_manifest_hash,
        workspace_manifest_hash(tmp.path())
    );
}

fn write_bytes(root: &std::path::Path, rel: &str, bytes: &[u8]) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, bytes).expect("write");
}

fn read_bytes(root: &std::path::Path, rel: &str) -> Vec<u8> {
    std::fs::read(root.join(rel)).expect("read")
}

#[test]
fn workspace_snapshot_restore_rewinds_dirty_tree() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    write_bytes(root, "keep.txt", b"original");
    write_bytes(root, "sub/nested.txt", b"nested");
    let checkpoint = ForkState::capture(root, "hist", "prev");

    write_bytes(root, "keep.txt", b"dirty");
    write_bytes(root, "new.txt", b"created during attempt");
    std::fs::remove_file(root.join("sub/nested.txt")).expect("rm nested");

    checkpoint.restore_workspace(root).expect("restore");

    assert_eq!(read_bytes(root, "keep.txt"), b"original");
    assert_eq!(read_bytes(root, "sub/nested.txt"), b"nested");
    assert!(!root.join("new.txt").exists());
}
