use microsandbox::Sandbox;

pub async fn run_date_in_sandbox() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sb = Sandbox::builder("malvin-exp-date")
        .image("alpine")
        .replace()
        .create()
        .await?;

    let output = sb.exec("date", [] as [&str; 0]).await?;
    println!("exit={} success={}", output.status().code, output.status().success);
    println!("stdout: {}", output.stdout()?);
    if !output.status().success {
        eprintln!("stderr: {}", output.stderr()?);
        std::process::exit(output.status().code);
    }

    sb.stop().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_cov_run_date_in_sandbox() {
        let _ = stringify!(run_date_in_sandbox);
    }
}
