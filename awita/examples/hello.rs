#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let window = awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await?;
    tokio::spawn(async move {
        while let Some(mouse) = window.on_mouse_input().await {
            println!("mouse_input: {:?}", mouse);
        }
    });
    awita::join().unwrap();
    Ok(())
}
