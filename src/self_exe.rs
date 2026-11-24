use std::path::PathBuf;

pub fn self_exe() -> std::io::Result<PathBuf> {
    std::env::current_exe().map(strip_deleted_suffix)
}

pub(crate) fn strip_deleted_suffix(exe: PathBuf) -> PathBuf {
    exe.to_str()
        .and_then(|s| s.strip_suffix(" (deleted)"))
        .map(PathBuf::from)
        .unwrap_or(exe)
}

pub(crate) fn parent_disconnected() -> tokio::sync::oneshot::Receiver<()> {
    let (closed, disconnected) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || {
        let _ = std::io::Read::read_to_end(&mut std::io::stdin(), &mut Vec::new());
        let _ = closed.send(());
    });
    disconnected
}
