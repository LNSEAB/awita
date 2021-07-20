#[tokio::main]
async fn main() {
    let _window = awita::window::Builder::new()
        .title("awita hello")
        .build()
        .await;
    awita::UiThread::get().join().unwrap();
}
