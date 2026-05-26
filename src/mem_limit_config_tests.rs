use super::*;

#[test]
fn parse_mem_limit_gb_reads_top_level_key() {
    let gb = parse_mem_limit_gb("mem_limit_gb = 2\n[logs]\nmax_runs = 1\n").expect("parse");
    assert_eq!(gb, 2);
}

#[test]
fn parse_mem_limit_gb_defaults_when_key_missing() {
    let gb = parse_mem_limit_gb("[logs]\nmax_runs = 1\n").expect("parse");
    assert_eq!(gb, default_mem_limit_gb());
}

#[test]
fn parse_mem_limit_gb_rejects_zero() {
    assert!(parse_mem_limit_gb("mem_limit_gb = 0\n").is_err());
}

#[test]
fn load_mem_limit_bytes_missing_config_uses_default() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bytes = load_mem_limit_bytes(tmp.path());
    assert_eq!(bytes, default_mem_limit_gb().saturating_mul(GIB));
}

#[test]
fn load_mem_limit_gb_reads_workspace_config() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = malvin_config_path(tmp.path());
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(&path, "mem_limit_gb = 6\n[logs]\nmax_runs = 1\n").expect("write");
    assert_eq!(load_mem_limit_gb(tmp.path()), 6);
}

#[test]
fn system_total_memory_bytes_positive_on_host() {
    let bytes = system_total_memory_bytes().expect("host mem");
    assert!(bytes >= GIB);
    #[cfg(target_os = "linux")]
    {
        assert!(linux_total_memory_bytes().is_some());
    }
    #[cfg(target_os = "macos")]
    {
        assert!(macos_total_memory_bytes().is_some());
    }
}
