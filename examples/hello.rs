#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let window = awita::Window::builder()
        .title("awita hello")
        .build()
        .await?;
    let mut closed = window.closed_receiver().await;
    loop {
        tokio::select! {
            Ok(_) = closed.recv() => {
                println!("closed");
            }
            _ = awita::UiThread::join() => break,
        }
    }
    awita::UiThread::maybe_unwind().await;
    Ok(())
}
