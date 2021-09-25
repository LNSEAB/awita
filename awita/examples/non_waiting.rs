#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let window = awita::Window::builder()
        .title("await non_waiting")
        .build()
        .await?;
    let mut resized = window.resized_receiver().await;
    let mut closed = window.closed_receiver().await;
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Ok(_) = closed.recv() => println!("closed"),
                _ = awita::UiThread::join() => break,
            }
        }
    });
    tokio::task::spawn_blocking(move || {
        while awita::UiThread::is_running() {
            if let Ok(Some(size)) = resized.try_recv() {
                println!("resized: ({}, {})", size.width, size.height);
            }
        }
    })
    .await?;
    awita::UiThread::maybe_unwind().await;
    Ok(())
}
