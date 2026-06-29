pub(crate) fn format_freed(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * KIB;
    if bytes >= MIB {
        format!("{} MiB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{} KiB", bytes / KIB)
    } else {
        format!("{bytes} B")
    }
}

pub(crate) fn format_max_bytes_display(max_bytes: Option<u64>) -> String {
    max_bytes.map_or_else(|| "unlimited".to_string(), format_freed)
}

pub(crate) fn format_max_count_display(max_count: u64) -> String {
    if max_count == 0 {
        "unlimited".to_string()
    } else {
        max_count.to_string()
    }
}

#[allow(dead_code)]
pub(crate) fn cached_total_bytes(sizes: &[u64]) -> u64 {
    sizes.iter().copied().sum()
}
