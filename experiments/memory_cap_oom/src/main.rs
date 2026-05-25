#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    memory_cap_oom::run_memory_cap_oom().await
}
