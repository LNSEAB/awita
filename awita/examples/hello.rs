#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let window = awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await?;
    let mut closed = window.closed_receiver().await;
    loop {
        tokio::select! {
            Ok(_) = closed.recv() => {
                println!("closed");
            }
            _ = awita::UiThread::finished() => break,
        }
    }
    awita::UiThread::resume_unwind().await;
    Ok(())
}
