use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ButtonState {
    Pressed,
    Released,
}

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
}

impl From<u32> for MouseButton {
    fn from(src: u32) -> Self {
        unsafe { std::mem::transmute(src) }
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
    pub fn contains(&self, button: MouseButton) -> bool {
        let button = button as u32;
        (self.0 & button) == button
    }
}

impl std::fmt::Debug for MouseButtons {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "MouseButtons([")?;
        let mut i = 0;
        while i < 32 {
            if self.0 & (1 << i) != 0 {
                write!(f, "{:?}", MouseButton::from(1 << i))?;
                i += 1;
                break;
            }
            i += 1;
        }
        while i < 32 {
            if self.0 & (1 << i) != 0 {
                write!(f, ", {:?}", MouseButton::from(1 << i))?;
            }
            i += 1;
        }
        write!(f, "])")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MouseState {
    pub position: PhysicalPoint<i32>,
    pub buttons: MouseButtons,
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
        assert!(MouseButton::Left == MouseButton::from(1u32 << 0));
        assert!(MouseButton::Right == MouseButton::from(1u32 << 1));
        assert!(MouseButton::Middle == MouseButton::from(1u32 << 2));
    }
}
