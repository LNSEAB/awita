#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await?;
    awita::finished().await;
    Ok(())
}
