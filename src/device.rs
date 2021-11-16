use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ButtonState {
    Pressed,
    Released,
}

pub type KeyState = ButtonState;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u32)]
pub enum MouseButton {
    Left = 1 << 0,
    Right = 1 << 1,
    Middle = 1 << 2,
    Ex0 = 1 << 3,
    Ex1 = 1 << 4,
    Ex2 = 1 << 5,
    Ex3 = 1 << 6,
    Ex4 = 1 << 7,
    Ex5 = 1 << 8,
    Ex6 = 1 << 9,
    Ex7 = 1 << 10,
    Ex8 = 1 << 11,
    Ex9 = 1 << 12,
    Ex10 = 1 << 13,
    Ex11 = 1 << 14,
    Ex12 = 1 << 15,
    Ex13 = 1 << 16,
    Ex14 = 1 << 17,
    Ex15 = 1 << 18,
    Ex16 = 1 << 19,
    Ex17 = 1 << 20,
    Ex18 = 1 << 21,
    Ex19 = 1 << 22,
    Ex20 = 1 << 23,
    Ex21 = 1 << 24,
    Ex22 = 1 << 25,
    Ex23 = 1 << 26,
    Ex24 = 1 << 27,
    Ex25 = 1 << 28,
    Ex26 = 1 << 29,
    Ex27 = 1 << 30,
    Ex28 = 1 << 31,
}

impl MouseButton {
    #[inline]
    pub fn ex(n: u32) -> Self {
        assert!(n <= 28);
        unsafe { std::mem::transmute(1u32 << (3 + n)) }
    }

    #[inline]
    fn from_u32(n: u32) -> Self {
        unsafe { std::mem::transmute(n) }
    }
}

#[derive(Clone, Copy)]
pub struct MouseButtons(u32);

impl MouseButtons {
    #[inline]
    pub fn new(buttons: &[MouseButton]) -> Self {
        Self(buttons.iter().fold(0, |bits, button| bits | *button as u32))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn contains(&self, button: MouseButton) -> bool {
        let button = button as u32;
        (self.0 & button) == button
    }

    #[inline]
    pub fn iter(&self) -> MouseButtonsIter {
        MouseButtonsIter {
            buttons: self,
            index: 0,
        }
    }
}

impl From<u32> for MouseButtons {
    #[inline]
    fn from(src: u32) -> Self {
        MouseButtons(src)
    }
}

impl std::fmt::Debug for MouseButtons {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MouseButtons([")?;
        let mut i = 0;
        while i < 32 {
            if self.0 & (1 << i) != 0 {
                write!(f, "{:?}", MouseButton::from_u32(1 << i))?;
                i += 1;
                break;
            }
            i += 1;
        }
        while i < 32 {
            if self.0 & (1 << i) != 0 {
                write!(f, ", {:?}", MouseButton::from_u32(1 << i))?;
            }
            i += 1;
        }
        write!(f, "])")
    }
}

#[derive(Clone)]
pub struct MouseButtonsIter<'a> {
    buttons: &'a MouseButtons,
    index: u32,
}

impl<'a> std::iter::Iterator for MouseButtonsIter<'a> {
    type Item = MouseButton;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 32 {
            return None;
        }
        let index = (self.index..32).find(|i| {
            let button = MouseButton::from_u32(1 << i);
            self.buttons.contains(button)
        })?;
        self.index = index + 1;
        Some(MouseButton::from_u32(1 << index))
    }
}

impl std::ops::BitOr<MouseButton> for MouseButton {
    type Output = MouseButtons;

    fn bitor(self, rhs: MouseButton) -> MouseButtons {
        MouseButtons(self as u32 | rhs as u32)
    }
}

impl std::ops::BitOr<MouseButton> for MouseButtons {
    type Output = MouseButtons;

    fn bitor(self, rhs: MouseButton) -> Self::Output {
        MouseButtons(self.0 | rhs as u32)
    }
}

impl std::ops::BitOr<MouseButtons> for MouseButton {
    type Output = MouseButtons;

    fn bitor(self, rhs: MouseButtons) -> MouseButtons {
        MouseButtons((self as u32) | rhs.0)
    }
}

impl std::ops::BitOr<MouseButtons> for MouseButtons {
    type Output = MouseButtons;

    fn bitor(self, rhs: MouseButtons) -> Self::Output {
        MouseButtons(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MouseState {
    pub position: PhysicalPoint<i32>,
    pub buttons: MouseButtons,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum VirtualKey {
    BackSpace = 0x08,
    Tab = 0x09,
    Enter = 0x0d,
    Shift = 0x10,
    Ctrl = 0x11,
    Alt = 0x12,
    Pause = 0x13,
    CapsLock = 0x14,
    Esc = 0x1b,
    Space = 0x20,
    PageUp = 0x21,
    PageDown = 0x22,
    End = 0x23,
    Home = 0x24,
    Left = 0x25,
    Up = 0x26,
    Right = 0x27,
    Down = 0x28,
    PrintScreen = 0x2c,
    Insert = 0x2d,
    Delete = 0x2e,
    _0 = 0x30,
    _1,
    _2,
    _3,
    _4,
    _5,
    _6,
    _7,
    _8,
    _9,
    A = 0x41,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    NumPad0 = 0x60,
    NumPad1,
    NumPad2,
    NumPad3,
    NumPad4,
    NumPad5,
    NumPad6,
    NumPad7,
    NumPad8,
    NumPad9,
    NumMul,
    NumAdd,
    NumDecimal,
    NumSub,
    NumDiv,
    F1 = 0x70,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    NumLock = 0x90,
    ScrollLock = 0x91,
    LShift = 0xa0,
    RShift,
    LCtrl = 0xa2,
    RCtrl,
    LAlt = 0xa4,
    RAlt,
}

impl VirtualKeyCode {
    #[inline]
    pub fn from_char(c: char) -> Option<Self> {
        if let Some(i) = ('0'..'9').position(|d| d == c) {
            Some(Self(VirtualKey::_0 as u32 + i as u32))
        } else if let Some(i) = ('A'..'Z').position(|d| d == c) {
            Some(Self(VirtualKey::A as u32 + i as u32))
        } else {
            None
        }
    }

    #[inline]
    pub fn f(n: u32) -> Option<Self> {
        (n >= 1 && n <= 24).then(|| Self(VirtualKey::F1 as u32 + n - 1))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VirtualKeyCode(pub u32);

#[derive(Clone, Copy, Debug)]
pub struct KeyCode {
    pub vkey: VirtualKeyCode,
    pub scan_code: u32,
}

impl PartialEq<VirtualKey> for VirtualKeyCode {
    #[inline]
    fn eq(&self, other: &VirtualKey) -> bool {
        match other {
            VirtualKey::Shift => *self == VirtualKey::LShift || *self == VirtualKey::RShift,
            VirtualKey::Ctrl => *self == VirtualKey::LCtrl || *self == VirtualKey::RCtrl,
            VirtualKey::Alt => *self == VirtualKey::LAlt || *self == VirtualKey::RAlt,
            _ => self.0 == *other as _,
        }
    }
}

impl PartialEq<VirtualKeyCode> for VirtualKey {
    #[inline]
    fn eq(&self, other: &VirtualKeyCode) -> bool {
        other == self
    }
}

impl PartialEq<VirtualKey> for KeyCode {
    #[inline]
    fn eq(&self, other: &VirtualKey) -> bool {
        self.vkey == *other
    }
}

impl PartialEq<KeyCode> for VirtualKey {
    #[inline]
    fn eq(&self, other: &KeyCode) -> bool {
        other == self
    }
}

impl From<VirtualKey> for VirtualKeyCode {
    #[inline]
    fn from(src: VirtualKey) -> Self {
        Self(src as _)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_ex() {
        for i in 0..=28 {
            assert!(MouseButton::ex(i) as u32 == 1u32 << (3 + i));
        }
    }

    #[test]
    fn u32_to_mouse_buton() {
        assert!(MouseButton::Left == MouseButton::from_u32(1u32 << 0));
        assert!(MouseButton::Right == MouseButton::from_u32(1u32 << 1));
        assert!(MouseButton::Middle == MouseButton::from_u32(1u32 << 2));
    }

    #[test]
    fn mouse_buttons_iter() {
        let buttons =
            MouseButtons::new(&[MouseButton::Left, MouseButton::Middle, MouseButton::Ex1]);
        let mut iter = buttons.iter();
        assert!(Some(MouseButton::Left) == iter.next());
        assert!(Some(MouseButton::Middle) == iter.next());
        assert!(Some(MouseButton::Ex1) == iter.next());
        assert!(None == iter.next());
    }

    #[test]
    fn mouse_button_bitor_mouse_button() {
        let ret = MouseButton::Left | MouseButton::Right;
        let mut iter = ret.iter();
        assert!(Some(MouseButton::Left) == iter.next());
        assert!(Some(MouseButton::Right) == iter.next());
        assert!(None == iter.next());
    }

    #[test]
    fn mouse_buttons_bitor_mouse_button() {
        let buttons = MouseButton::Left | MouseButton::Right;
        let ret = buttons | MouseButton::Middle;
        let mut iter = ret.iter();
        assert!(Some(MouseButton::Left) == iter.next());
        assert!(Some(MouseButton::Right) == iter.next());
        assert!(Some(MouseButton::Middle) == iter.next());
        assert!(None == iter.next());
    }

    #[test]
    fn mouse_button_bitor_mouse_buttons() {
        let buttons = MouseButton::Left | MouseButton::Right;
        let ret = MouseButton::Middle | buttons;
        let mut iter = ret.iter();
        assert!(Some(MouseButton::Left) == iter.next());
        assert!(Some(MouseButton::Right) == iter.next());
        assert!(Some(MouseButton::Middle) == iter.next());
        assert!(None == iter.next());
    }

    #[test]
    fn mouse_buttons_bitor_mouse_buttons() {
        let b0 = MouseButton::Left | MouseButton::Right;
        let b1 = MouseButton::Middle | MouseButton::Ex28;
        let ret = b0 | b1;
        let mut iter = ret.iter();
        assert!(Some(MouseButton::Left) == iter.next());
        assert!(Some(MouseButton::Right) == iter.next());
        assert!(Some(MouseButton::Middle) == iter.next());
        assert!(Some(MouseButton::Ex28) == iter.next());
        assert!(None == iter.next());
    }

    #[test]
    fn virtual_key_code_from_char() {
        for (i, c) in ('0'..'9').enumerate() {
            let fc = VirtualKeyCode::from_char(c).unwrap();
            assert!(fc == VirtualKeyCode(VirtualKey::_0 as u32 + i as u32));
        }
        for (i, c) in ('A'..'Z').enumerate() {
            let fc = VirtualKeyCode::from_char(c).unwrap();
            assert!(fc == VirtualKeyCode(VirtualKey::A as u32 + i as u32));
        }
        assert!(VirtualKeyCode::from_char('!').is_none());
    }

    #[test]
    fn f_keys() {
        for i in 1u32..24 {
            let f = VirtualKeyCode::f(i).unwrap();
            assert!(f == VirtualKeyCode(VirtualKey::F1 as u32 + i - 1));
        }
        assert!(VirtualKeyCode::f(25).is_none());
    }

    #[test]
    fn key_code_eq() {
        assert!(VirtualKeyCode(VirtualKey::LShift as u32) == VirtualKey::Shift);
        assert!(VirtualKeyCode(VirtualKey::RShift as u32) == VirtualKey::Shift);
        assert!(VirtualKeyCode(VirtualKey::LCtrl as u32) == VirtualKey::Ctrl);
        assert!(VirtualKeyCode(VirtualKey::RCtrl as u32) == VirtualKey::Ctrl);
        assert!(VirtualKeyCode(VirtualKey::LAlt as u32) == VirtualKey::Alt);
        assert!(VirtualKeyCode(VirtualKey::RAlt as u32) == VirtualKey::Alt);
    }

    #[test]
    fn button_state_and_key_state() {
        assert!(ButtonState::Pressed == KeyState::Pressed);
    }
}
