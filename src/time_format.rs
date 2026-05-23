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
    #[test]
    fn kiss_cov_timestamp_now_string() {
        let _ = super::timestamp_now_string;
    }
}
