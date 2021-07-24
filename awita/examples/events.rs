#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let window = awita::window::Builder::new()
        .title("awita hello")
        .accept_drop_files(true)
        .build()
        .await?;
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
            data = cursor_entered.recv() => {
                if let Some(data) = data {
                    println!("cursor_entered: {:?}", data);
                }
            }
            data = cursor_leaved.recv() => {
                if let Some(data) = data {
                    println!("cursor_leaved: {:?}", data);
                }
            }
            data = cursor_moved.recv() => {
                if let Some(data) = data {
                    println!("cursor_moved: {:?}", data);
                }
            }
            data = mouse_input.recv() => {
                if let Some(data) = data {
                    println!("mouse_input: {:?}", data);
                }
            }
            data = key_input.recv() => {
                if let Some(data) = data {
                    println!("key_input: {:?}", data);
                }
            }
            c = char_input.recv() => {
                if let Some(c) = c {
                    if c.is_ascii_control() {
                        println!("char_input: 0x{:x}", c as u32);
                    } else {
                        println!("char_input: {}", c);
                    }
                }
            }
            data = moved.recv() => {
                if let Some(data) = data {
                    println!("moved: {:?}", data);
                }
            }
            data = sizing.recv() => {
                if let Some(data) = data {
                    println!("sizing: {:?}", data);
                }
            }
            data = sized.recv() => {
                if let Some(data) = data {
                    println!("sized: {:?}", data);
                }
            }
            v = activated.recv() => {
                v.is_some().then(|| println!("activated"));
            }
            v = inactivated.recv() => {
                v.is_some().then(|| println!("inactivated"));
            }
            dpi = dpi_changed.recv() => {
                if let Some(dpi) = dpi {
                    println!("dpi_changed: {}", dpi);
                }
            }
            data = drop_files.recv() => {
                if let Some(data) = data {
                    println!("drop_files: {:?}", data);
                }
            }
            close_req = close_request.recv() => {
                if let Some(close_req) = close_req {
                    println!("close_request");
                    close_req.close();
                }
            }
            v = closed.recv() => {
                v.is_some().then(|| println!("closed"));
            }
            _ = awita::finished() => break,
        }
    }
    Ok(())
}
