use super::*;

#[derive(Clone, Copy, Debug)]
pub struct MouseInput {
    pub button: MouseButton,
    pub button_state: ButtonState,
    pub mouse_state: MouseState,
}
