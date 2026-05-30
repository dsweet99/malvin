use super::*;
use crate::malvin_config_path;

#[test]
fn parse_mem_limit_gb_reads_top_level_key() {
    let gb = parse_mem_limit_gb("mem_limit_gb = 2\n[logs]\nmax_age_days = 1\n").expect("parse");
    assert_eq!(gb, 2);
}

#[test]
fn parse_mem_limit_gb_defaults_when_key_missing() {
    let gb = parse_mem_limit_gb("[logs]\nmax_age_days = 1\n").expect("parse");
    assert_eq!(gb, default_mem_limit_gb());
}

#[test]
fn parse_mem_limit_gb_rejects_zero() {
    assert!(parse_mem_limit_gb("mem_limit_gb = 0\n").is_err());
}

#[test]
fn load_mem_limit_bytes_missing_config_uses_default() {
    crate::test_utils::with_isolated_home(|work| {
        let bytes = load_mem_limit_bytes(work);
        assert_eq!(bytes, default_mem_limit_gb().saturating_mul(GIB));
    });
}

#[test]
fn load_mem_limit_gb_reads_home_config() {
    crate::test_utils::with_isolated_home(|work| {
        let path = malvin_config_path(work);
        std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        std::fs::write(&path, "mem_limit_gb = 6\n[logs]\nmax_age_days = 1\n").expect("write");
        assert_eq!(load_mem_limit_gb(work), 6);
    });
}

#[test]
fn format_host_resources_line_mentions_memory_and_cpus() {
    let line = format_host_resources_line();
    assert!(line.starts_with("Memory: "));
    assert!(line.contains(", CPUs: "));
    let cpus = system_cpu_count().expect("cpus");
    assert!(cpus > 0);
    assert!(line.contains(&cpus.to_string()));
}

#[test]
fn format_memory_gib_rounds_near_whole_gib() {
    assert_eq!(format_memory_gib(GIB), "1 GiB");
    assert_eq!(format_memory_gib(2 * GIB), "2 GiB");
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
