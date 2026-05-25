#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    date_in_sandbox::run_date_in_sandbox().await
}
