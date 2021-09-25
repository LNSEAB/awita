# awita

[![awita at crates.io](https://img.shields.io/crates/v/awita.svg)](https://crates.io/crates/awita)
[![awita at docs.rs](https://docs.rs/awita/badge.svg)](https://docs.rs/awita)

An asynchronous window library for Windows

## Overview

"awita" is an asynchronous window creation and management library for Windows.
A window event can be received asynchronously using a receiver.

## Examples

### Waiting event loop

```rust
#[tokio::main]
async main() {
	let window = awita::Window::builder()
        .title("awita hello")
        .build()
        .await
		.unwrap();
    let mut closed = window.closed_receiver().await;
	loop {
		tokio::select! {
			Ok(_) = closed.recv() => println!("closed"),
			_ = awita::UiThread::join() => break,
		}
	}
	awita::UiThread::maybe_unwind().await;
}
```

### Non-waiting event loop

```rust
#[tokio::main]
async fn main() {
	let window = awita::Window::builder()
		.title("await non_waiting")
		.build()
		.await
		.unwrap();
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
			// For example, write a rendering code.

			if let Ok(Some(size)) = resized.try_recv() {
				println!("resized: ({}, {})", size.width, size.height);
			}
		}
	})
	.await?;
	awita::UiThread::maybe_unwind().await;
}
```

## License

[MIT license](LICENSE)

-----------------------------------------------------------------------

Copyright (c) 2021 LNSEAB
