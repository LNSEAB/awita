#[tokio::main]
async fn main() -> anyhow::Result<()> {
    awita::window::Builder::new()
        .title("awita icon")
        .icon(awita::Icon::File("examples/icon.ico".into()))
        .build()
        .await?;
    awita::UiThread::join().await;
    awita::UiThread::maybe_unwind().await;
    Ok(())
}
