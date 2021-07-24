use super::*;
use tokio::sync::broadcast;

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

#[derive(Clone, Debug)]
pub struct DropFiles {
    pub position: PhysicalPoint<i32>,
    pub files: Vec<std::path::PathBuf>,
}
