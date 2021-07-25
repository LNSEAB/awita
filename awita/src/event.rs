use super::*;
use tokio::sync::{broadcast, mpsc};
use awita_windows_bindings::Windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::*,
};

pub struct Receiver<R>(pub(crate) Option<broadcast::Receiver<R>>);

impl<R> Receiver<R>
where
    R: Clone,
{
    pub async fn recv(&mut self) -> Option<R> {
        if self.0.is_none() {
            return None;
        }
        let rx = self.0.as_mut().unwrap();
        rx.recv().await.ok()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MouseInput {
    pub button: MouseButton,
    pub button_state: ButtonState,
    pub mouse_state: MouseState,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyInput {
    pub state: ButtonState,
    pub key_code: KeyCode,
    pub prev_state: ButtonState,
}

#[derive(Debug)]
pub struct CloseRequest(pub(crate) HWND);

impl CloseRequest {
    #[inline]
    pub fn close(self) {
        UiThread::post_with_context(move |_| unsafe {
            DestroyWindow(self.0);
        });
    }
}

#[derive(Debug)]
pub struct CloseRequestReceiver {
    rx: mpsc::Receiver<CloseRequest>,
}

impl CloseRequestReceiver {
    pub(crate) fn new(rx: mpsc::Receiver<CloseRequest>) -> Self {
        Self {
            rx
        }
    }

    #[inline]
    pub async fn recv(&mut self) -> Option<CloseRequest> {
        self.rx.recv().await
    }
}

#[derive(Clone, Debug)]
pub struct DropFiles {
    pub position: PhysicalPoint<i32>,
    pub files: Vec<std::path::PathBuf>,
}
