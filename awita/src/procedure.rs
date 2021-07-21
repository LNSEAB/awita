use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    UI::{HiDpi::*, WindowsAndMessaging::*},
};
use std::rc::Rc;

fn context() -> Rc<Context> {
    CONTEXT.with(|ctx| ctx.borrow().as_ref().unwrap().clone())
}

fn loword(x: i32) -> i16 {
    (x & 0xffff) as _
}

fn hiword(x: i32) -> i16 {
    ((x >> 16) & 0xffff) as _
}

fn get_x_lparam(lp: LPARAM) -> i16 {
    loword(lp.0 as _)
}

fn get_y_lparam(lp: LPARAM) -> i16 {
    hiword(lp.0 as _)
}

fn get_xbutton_wparam(wp: WPARAM) -> i16 {
    hiword(wp.0 as _)
}

fn get_button_states(wp: WPARAM) -> u32 {
    loword(wp.0 as _) as _
}

fn lparam_to_point(lp: LPARAM) -> PhysicalPoint<i32> {
    Physical(Point::new(get_x_lparam(lp) as _, get_y_lparam(lp) as _))
}

fn xbutton_to_ex(wp: WPARAM) -> MouseButton {
    let n = get_xbutton_wparam(wp) as u32;
    MouseButton::ex(n - 1)
}

fn get_mouse_buttons(wparam: WPARAM) -> MouseButtons {
    let mut buttons = 0;
    let values = get_button_states(wparam);
    if values & MK_LBUTTON != 0 {
        buttons |= MouseButton::Left as u32;
    }
    if values & MK_RBUTTON != 0 {
        buttons |= MouseButton::Right as u32;
    }
    if values & MK_MBUTTON != 0 {
        buttons |= MouseButton::Middle as u32;
    }
    if values & MK_XBUTTON1 != 0 {
        buttons |= MouseButton::Ex0 as u32;
    }
    if values & MK_XBUTTON2 != 0 {
        buttons |= MouseButton::Ex1 as u32;
    }
    buttons.into()
}

unsafe fn mouse_input(
    hwnd: HWND,
    button: MouseButton,
    button_state: ButtonState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(window) = context().get_window(hwnd) {
        let mouse_state = MouseState {
            position: lparam_to_point(lparam),
            buttons: get_mouse_buttons(wparam),
        };
        window
            .mouse_input_tx
            .send(event::MouseInput {
                button,
                button_state,
                mouse_state,
            })
            .ok();
    }
    LRESULT(0)
}

unsafe fn wm_dpi_changed(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let rc = *(lparam.0 as *const RECT);
    SetWindowPos(
        hwnd,
        HWND::NULL,
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
    LRESULT(0)
}

unsafe fn wm_get_dpi_scaled_size(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let prev_dpi = GetDpiForWindow(hwnd) as i32;
    let next_dpi = wparam.0 as i32;
    let mut rc = RECT::default();
    GetClientRect(hwnd, &mut rc);
    let size = Physical(Size::new(
        (rc.right * next_dpi / prev_dpi) as u32,
        (rc.bottom * next_dpi / prev_dpi) as u32,
    ));
    let size = utility::adjust_window_size(
        size,
        WINDOW_STYLE(GetWindowLongPtrW(hwnd, GWL_STYLE) as _),
        WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as _),
        next_dpi as _,
    );
    let mut ret = (lparam.0 as *mut SIZE).as_mut().unwrap();
    ret.cx = size.width as _;
    ret.cy = size.height as _;
    LRESULT(1)
}

unsafe fn wm_activate(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    if let Some(window) = context().get_window(hwnd) {
        if (wparam.0 as u32 & WA_ACTIVE) != 0 || (wparam.0 as u32 & WA_CLICKACTIVE) != 0 {
            window.activated_tx.send(()).ok();
        } else {
            window.inactivated_tx.send(()).ok();
        }
    }
    LRESULT(0)
}

unsafe fn wm_destroy(hwnd: HWND) -> LRESULT {
    if let Some(window) = context().get_window(hwnd) {
        window.closed_tx.send(()).ok();
    }
    context().remove_window(hwnd);
    if context().window_map_is_empty() {
        PostQuitMessage(0);
    }
    LRESULT(0)
}

unsafe fn wm_nc_create(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    EnableNonClientDpiScaling(hwnd);
    DefWindowProcW(hwnd, WM_NCCREATE, wparam, lparam)
}

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ret = std::panic::catch_unwind(|| match msg {
        WM_LBUTTONDOWN => mouse_input(
            hwnd,
            MouseButton::Left,
            ButtonState::Pressed,
            wparam,
            lparam,
        ),
        WM_RBUTTONDOWN => mouse_input(
            hwnd,
            MouseButton::Right,
            ButtonState::Pressed,
            wparam,
            lparam,
        ),
        WM_MBUTTONDOWN => mouse_input(
            hwnd,
            MouseButton::Middle,
            ButtonState::Pressed,
            wparam,
            lparam,
        ),
        WM_XBUTTONDOWN => mouse_input(
            hwnd,
            xbutton_to_ex(wparam),
            ButtonState::Pressed,
            wparam,
            lparam,
        ),
        WM_LBUTTONUP => mouse_input(
            hwnd,
            MouseButton::Left,
            ButtonState::Released,
            wparam,
            lparam,
        ),
        WM_RBUTTONUP => mouse_input(
            hwnd,
            MouseButton::Right,
            ButtonState::Released,
            wparam,
            lparam,
        ),
        WM_MBUTTONUP => mouse_input(
            hwnd,
            MouseButton::Middle,
            ButtonState::Released,
            wparam,
            lparam,
        ),
        WM_XBUTTONUP => mouse_input(
            hwnd,
            xbutton_to_ex(wparam),
            ButtonState::Released,
            wparam,
            lparam,
        ),
        WM_DPICHANGED => wm_dpi_changed(hwnd, lparam),
        WM_GETDPISCALEDSIZE => wm_get_dpi_scaled_size(hwnd, wparam, lparam),
        WM_ACTIVATE => wm_activate(hwnd, wparam),
        WM_DESTROY => wm_destroy(hwnd),
        WM_NCCREATE => wm_nc_create(hwnd, wparam, lparam),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    });
    match ret {
        Ok(ret) => ret,
        Err(e) => {
            context().set_unwind(e);
            LRESULT(0)
        }
    }
}
