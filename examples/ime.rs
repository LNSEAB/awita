#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let window = awita::Window::builder()
        .title("awita ime")
        .enable_ime(true)
        .visible_ime_candidate_window(false)
        .build()
        .await?;
    let mut ime_start_composition = window.ime_start_composition_receiver().await;
    let mut ime_composition = window.ime_composition_receiver().await;
    let mut ime_end_composition = window.ime_end_composition_receiver().await;
    loop {
        tokio::select! {
            Ok(_) = ime_start_composition.recv() => {
                println!("ime_start_composition");
            }
            Ok(ret) = ime_composition.recv() => {
                println!("ime_composition: {:?}", ret);
            }
            Ok(ret) = ime_end_composition.recv() => {
                println!("ime_end_composition: {:?}", ret);
            }
            _ = awita::UiThread::join() => break,
        }
    }
    awita::UiThread::maybe_unwind().await;
    Ok(())
}
