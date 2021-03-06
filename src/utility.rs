use super::*;
use windows::Win32::{
    Foundation::*,
    UI::{HiDpi::*, WindowsAndMessaging::*},
};

pub fn adjust_window_size(
    size: PhysicalSize<u32>,
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
    dpi: u32,
) -> PhysicalSize<u32> {
    unsafe {
        let mut rc = RECT {
            left: 0,
            top: 0,
            right: size.width as _,
            bottom: size.height as _,
        };
        AdjustWindowRectExForDpi(&mut rc, style, false, ex_style, dpi);
        Physical(Size::new(
            (rc.right - rc.left) as _,
            (rc.bottom - rc.top) as _,
        ))
    }
}
