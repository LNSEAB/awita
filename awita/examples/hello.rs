#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let window = awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await?;
    window.closed_receiver().await.recv().await.unwrap();
    Ok(())
}
