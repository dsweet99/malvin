#[must_use]
pub fn timestamp_now_string() -> String {
    let now = chrono::Local::now();
    format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    )
}

#[cfg(test)]
mod tests {
    use super::timestamp_now_string;

    #[test]
    fn timestamp_now_string_nonempty() {
        assert!(!timestamp_now_string().is_empty());
    }
}
