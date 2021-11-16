use super::*;
use tokio::sync::mpsc;
use windows::Win32::{Foundation::HWND, UI::WindowsAndMessaging::*};

#[derive(Clone)]
pub struct Receiver<R>(pub(crate) Option<async_broadcast::Receiver<R>>);

impl<R> Receiver<R>
where
    R: Clone,
{
    pub async fn recv(&mut self) -> Result<R, Error> {
        if self.0.is_none() {
            return Err(Error::Closed);
        }
        let rx = self.0.as_mut().unwrap();
        rx.recv().await.map_err(|_| Error::Closed)
    }

    pub fn try_recv(&mut self) -> Result<Option<R>, Error> {
        if self.0.is_none() {
            return Err(Error::Closed);
        }
        let rx = self.0.as_mut().unwrap();
        match rx.try_recv() {
            Ok(r) => Ok(Some(r)),
            Err(e) => match e {
                async_broadcast::TryRecvError::Empty => Ok(None),
                async_broadcast::TryRecvError::Closed => Err(Error::Closed),
            },
        }
    }
}

pub(crate) struct Channel<T> {
    pub tx: async_broadcast::Sender<T>,
    pub rx: async_broadcast::InactiveReceiver<T>,
}

impl<T> Channel<T>
where
    T: Clone,
{
    #[inline]
    pub fn new(capacity: usize) -> Self {
        let (mut tx, rx) = async_broadcast::broadcast(capacity);
        tx.set_overflow(true);
        Self {
            tx,
            rx: rx.deactivate(),
        }
    }

    #[inline]
    pub fn send(&self, value: T) {
        self.tx.try_broadcast(value).ok();
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MouseInput {
    pub button: MouseButton,
    pub button_state: ButtonState,
    pub mouse_state: MouseState,
}

#[derive(Clone, Copy, Debug)]
pub struct MouseWheel {
    pub delta: i16,
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
        Self { rx }
    }

    #[inline]
    pub async fn recv(&mut self) -> Result<CloseRequest, Error> {
        self.rx.recv().await.ok_or(Error::Closed)
    }
}

#[derive(Clone, Debug)]
pub struct DropFiles {
    pub position: PhysicalPoint<i32>,
    pub files: Vec<std::path::PathBuf>,
}
