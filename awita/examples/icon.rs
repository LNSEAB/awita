#[tokio::main]
async fn main() -> anyhow::Result<()> {
    awita::window::Builder::new()
        .title("awita icon")
        .icon(awita::Icon::File("examples/icon.ico".into()))
        .build()
        .await?;
    awita::UiThread::finished().await;
    awita::UiThread::resume_unwind().await;
    Ok(())
}
