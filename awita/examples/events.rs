#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let window = awita::window::Builder::new()
        .title("awita events")
        .accept_drop_files(true)
        .build()
        .await?;
    let mut draw = window.draw_receiver().await;
    let mut cursor_entered = window.cursor_entered_receiver().await;
    let mut cursor_leaved = window.cursor_leaved_receiver().await;
    let mut cursor_moved = window.cursor_moved_receiver().await;
    let mut mouse_input = window.mouse_input_receiver().await;
    let mut key_input = window.key_input_receiver().await;
    let mut char_input = window.char_input_receiver().await;
    let mut moved = window.moved_receiver().await;
    let mut sizing = window.sizing_receiver().await;
    let mut sized = window.sized_receiver().await;
    let mut activated = window.activated_receiver().await;
    let mut inactivated = window.inactivated_receiver().await;
    let mut dpi_changed = window.dpi_changed_receiver().await;
    let mut close_request = window.close_request_receiver().await;
    let mut drop_files = window.drop_files_receiver().await;
    let mut closed = window.closed_receiver().await;
    loop {
        tokio::select! {
            Ok(_) = draw.recv() => {
                println!("draw");
            }
            Ok(data) = cursor_entered.recv() => {
                println!("cursor_entered: {:?}", data);
            }
            Ok(data) = cursor_leaved.recv() => {
                println!("cursor_leaved: {:?}", data);
            }
            Ok(data) = cursor_moved.recv() => {
                println!("cursor_moved: {:?}", data);
            }
            Ok(data) = mouse_input.recv() => {
                println!("mouse_input: {:?}", data);
            }
            Ok(data) = key_input.recv() => {
                println!("key_input: {:?}", data);
            }
            Ok(c) = char_input.recv() => {
                if c.is_ascii_control() {
                    println!("char_input: 0x{:x}", c as u32);
                } else {
                    println!("char_input: {}", c);
                }
            }
            Ok(data) = moved.recv() => {
                println!("moved: {:?}", data);
            }
            Ok(data) = sizing.recv() => {
                println!("sizing: {:?}", data);
            }
            Ok(data) = sized.recv() => {
                println!("sized: {:?}", data);
            }
            Ok(_) = activated.recv() => {
                println!("activated");
            }
            Ok(_) = inactivated.recv() => {
                println!("inactivated");
            }
            Ok(dpi) = dpi_changed.recv() => {
                println!("dpi_changed: {}", dpi);
            }
            Ok(data) = drop_files.recv() => {
                println!("drop_files: {:?}", data);
            }
            Ok(close_req) = close_request.recv() => {
                println!("close_request");
                close_req.close();
            }
            Ok(_) = closed.recv() => {
                println!("closed");
            }
            _ = awita::UiThread::finished() => break,
        }
    }
    awita::UiThread::resume_unwind().await;
    Ok(())
}
