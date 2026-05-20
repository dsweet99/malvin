#[cfg(unix)]
pub fn signal_process_group(process_group_id: u32, signal: i32) {
    let Ok(pgid) = i32::try_from(process_group_id) else {
        return;
    };
    let target = format!("-{pgid}");
    let signal = format!("-{signal}");
    let _ = std::process::Command::new("kill")
        .arg(signal)
        .arg("--")
        .arg(target)
        .stderr(std::process::Stdio::null())
        .status();
}

#[cfg(unix)]
pub async fn terminate_process_group(process_group_id: Option<u32>) {
    let Some(process_group_id) = process_group_id else {
        return;
    };
    signal_process_group(process_group_id, 15);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    signal_process_group(process_group_id, 9);
}

#[cfg(not(unix))]
pub async fn terminate_process_group(_: Option<u32>) {}

#[cfg(test)]
mod kiss_coverage {
    #[tokio::test]
    async fn terminate_process_group_none_is_noop() {
        #[cfg(unix)]
        let _ = super::signal_process_group;
        super::terminate_process_group(None).await;
    }
}
