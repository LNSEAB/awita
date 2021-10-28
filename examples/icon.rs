#[tokio::main]
async fn main() -> anyhow::Result<()> {
    awita::Window::builder()
        .title("awita icon")
        .icon(awita::Icon::File("examples/icon.ico".into()))
        .build()
        .await?;
    awita::UiThread::join().await;
    awita::UiThread::maybe_unwind().await;
    Ok(())
}
