use std::path::Path;

pub(super) fn visit_source_files(root: &Path, f: &mut impl FnMut(&Path)) {
    fn walk(dir: &Path, f: &mut impl FnMut(&Path)) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_symlink() {
                continue;
            }
            let path = entry.path();
            if file_type.is_file() {
                f(&path);
            } else if file_type.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "target" || name == "__pycache__" {
                        continue;
                    }
                }
                walk(&path, f);
            }
        }
    }
    walk(root, f);
}

pub(super) fn python_ruff_and_pytest_flags(root: &Path) -> (bool, bool) {
    let mut has_py = false;
    let mut has_pytest = false;
    visit_source_files(root, &mut |path: &Path| {
        if path.extension().and_then(|e| e.to_str()) != Some("py") {
            return;
        }
        has_py = true;
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };
        if stem.starts_with("test_") || stem.ends_with("_test") {
            has_pytest = true;
        }
    });
    (has_py, has_pytest)
}
