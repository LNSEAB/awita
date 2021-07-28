#[tokio::main]
async fn main() -> anyhow::Result<()> {
    awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await?;
    awita::UiThread::finished().await;
    awita::UiThread::resume_unwind().await;
    Ok(())
}
