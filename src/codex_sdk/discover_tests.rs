use super::discover::path_is_executable;

#[test]
fn codex_path_is_executable_is_covered() {
    let _ = path_is_executable;
    assert!(path_is_executable(std::path::Path::new(".")));
}
