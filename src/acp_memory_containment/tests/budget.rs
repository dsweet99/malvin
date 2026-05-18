#[cfg(target_os = "linux")]
mod linux {
    use crate::acp_memory_containment::half_physical_memory_bytes;

    #[test]
    fn half_physical_memory_is_positive_on_linux() {
        let half = half_physical_memory_bytes().expect("linux meminfo");
        assert!(half > 0);
    }

    #[test]
    fn linux_half_memory_matches_memtotal() {
        let meminfo = std::fs::read_to_string("/proc/meminfo").expect("meminfo");
        let total_kb = meminfo
            .lines()
            .find_map(|line| {
                let rest = line.strip_prefix("MemTotal:")?;
                rest.split_whitespace().next()?.parse::<u64>().ok()
            })
            .expect("MemTotal");
        let expected = total_kb * 1024 / 2;
        assert_eq!(
            crate::acp_memory_containment::half_physical_memory_bytes(),
            Some(expected)
        );
    }
}

#[cfg(not(target_os = "linux"))]
mod non_linux {
    use crate::acp_memory_containment::half_physical_memory_bytes;

    #[test]
    fn half_physical_memory_unavailable_off_linux() {
        assert!(half_physical_memory_bytes().is_none());
    }
}
