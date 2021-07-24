use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    UI::{Controls::*, HiDpi::*, KeyboardAndMouseInput::*, Shell::*, WindowsAndMessaging::*},
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

unsafe fn wm_mouse_move(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let context = context();
    let window = match context.get_window(hwnd) {
        Some(window) => window,
        None => return DefWindowProcW(hwnd, WM_MOUSEMOVE, wparam, lparam),
    };
    let state = MouseState {
        position: lparam_to_point(lparam),
        buttons: get_mouse_buttons(wparam),
    };
    if context.entered_cursor_window.get().is_none() {
        TrackMouseEvent(&mut TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as _,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            ..Default::default()
        });
        context.entered_cursor_window.set(Some(hwnd));
        window.cursor_entered_channel.send(state).ok();
    } else {
        window.cursor_moved_chennel.send(state).ok();
    }
    LRESULT(0)
}

unsafe fn wm_mouse_leave(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let context = context();
    let window = match context.get_window(hwnd) {
        Some(window) => window,
        None => return DefWindowProcW(hwnd, WM_MOUSELEAVE, wparam, lparam),
    };
    let mut position = POINT::default();
    GetCursorPos(&mut position);
    ScreenToClient(hwnd, &mut position);
    context.entered_cursor_window.set(None);
    window
        .cursor_leaved_channel
        .send(MouseState {
            position: Physical(Point::new(position.x, position.y)),
            buttons: get_mouse_buttons(wparam),
        })
        .ok();
    LRESULT(0)
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
            .mouse_input_channel
            .send(event::MouseInput {
                button,
                button_state,
                mouse_state,
            })
            .ok();
    }
    LRESULT(0)
}

unsafe fn key_input(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let context = context();
    let window = match context.get_window(hwnd) {
        Some(window) => window,
        None => return DefWindowProcW(hwnd, msg, wparam, lparam),
    };
    let state = match msg {
        WM_KEYDOWN | WM_SYSKEYDOWN => ButtonState::Pressed,
        WM_KEYUP | WM_SYSKEYUP => ButtonState::Released,
        _ => unreachable!(),
    };
    let scan_code = ((lparam.0 >> 16) & 0xff) as u32;
    let extended = (lparam.0 >> 24) & 0x01 != 0;
    let vkey = match wparam.0 as u32 {
        VK_SHIFT => MapVirtualKeyW(scan_code, MAPVK_VSC_TO_VK_EX),
        VK_CONTROL => {
            if extended {
                VK_RCONTROL
            } else {
                VK_LCONTROL
            }
        }
        VK_MENU => {
            if extended {
                VK_RMENU
            } else {
                VK_LMENU
            }
        }
        v => v,
    };
    let key_code = KeyCode {
        vkey: VirtualKeyCode(vkey),
        scan_code,
    };
    let prev_state = if (lparam.0 >> 30) & 0x01 != 0 {
        ButtonState::Pressed
    } else {
        ButtonState::Released
    };
    window
        .key_input_channel
        .send(event::KeyInput {
            state,
            key_code,
            prev_state,
        })
        .ok();
    LRESULT(0)
}

unsafe fn wm_char(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    if let Some(window) = context().get_window(hwnd) {
        if let Some(c) = char::from_u32(wparam.0 as _) {
            window.char_input_channel.send(c).ok();
        }
    }
    LRESULT(0)
}

unsafe fn wm_move(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let context = context();
    let window = match context.get_window(hwnd) {
        Some(window) => window,
        None => return LRESULT(0),
    };
    let position = lparam_to_point(lparam);
    window
        .moved_channel
        .send(Screen(Point::new(position.x, position.y)))
        .ok();
    LRESULT(0)
}

unsafe fn wm_size(hwnd: HWND, lparam: LPARAM) -> LRESULT {
    let value = lparam.0 as i32;
    let size = Physical(Size::new(loword(value) as u32, hiword(value) as u32));
    if let Some(window) = context().get_window(hwnd) {
        window.sizing_channel.send(size).ok();
    }
    LRESULT(0)
}

unsafe fn wm_enter_size_move(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    context().resizing.set(true);
    DefWindowProcW(hwnd, WM_ENTERSIZEMOVE, wparam, lparam)
}

unsafe fn wm_exit_size_move(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    context().resizing.set(false);
    let mut rc = RECT::default();
    GetClientRect(hwnd, &mut rc);
    if let Some(window) = context().get_window(hwnd) {
        window
            .sized_channel
            .send(Physical(Size::new(rc.right as u32, rc.bottom as u32)))
            .ok();
    }
    DefWindowProcW(hwnd, WM_EXITSIZEMOVE, wparam, lparam)
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
    let dpi = GetDpiForWindow(hwnd);
    if let Some(window) = context().get_window(hwnd) {
        window.dpi_changed_channel.send(dpi).ok();
    }
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
        let value = loword(wparam.0 as _) as u32;
        if value == 0 {
            window.inactivated_channel.send(()).ok();
        } else {
            window.activated_channel.send(()).ok();
        }
    }
    LRESULT(0)
}

unsafe fn wm_drop_files(hwnd: HWND, wparam: WPARAM) -> LRESULT {
    let context = context();
    let window = match context.get_window(hwnd) {
        Some(window) => window,
        None => return LRESULT(0),
    };
    let hdrop = HDROP(wparam.0 as _);
    let count = DragQueryFileW(hdrop, u32::MAX, PWSTR::NULL, 0);
    let mut buffer = vec![];
    let mut files: Vec<std::path::PathBuf> = Vec::with_capacity(count as usize);
    for i in 0..count {
        let len = DragQueryFileW(hdrop, i, PWSTR::NULL, 0) as usize + 1;
        buffer.resize(len, 0);
        DragQueryFileW(hdrop, i, PWSTR(buffer.as_mut_ptr()), len as u32);
        buffer.pop();
        files.push(String::from_utf16_lossy(&buffer).into());
    }
    let mut pt = POINT::default();
    DragQueryPoint(hdrop, &mut pt);
    window
        .drop_files_channel
        .send(event::DropFiles {
            position: Physical(Point::new(pt.x, pt.y)),
            files,
        })
        .ok();
    LRESULT(0)
}

unsafe fn wm_destroy(hwnd: HWND) -> LRESULT {
    if let Some(window) = context().get_window(hwnd) {
        window.closed_channel.send(()).ok();
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
        WM_MOUSEMOVE => wm_mouse_move(hwnd, wparam, lparam),
        WM_MOUSELEAVE => wm_mouse_leave(hwnd, wparam, lparam),
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
        WM_KEYDOWN => key_input(hwnd, WM_KEYDOWN, wparam, lparam),
        WM_KEYUP => key_input(hwnd, WM_KEYUP, wparam, lparam),
        WM_SYSKEYDOWN => key_input(hwnd, WM_SYSKEYDOWN, wparam, lparam),
        WM_SYSKEYUP => key_input(hwnd, WM_SYSKEYDOWN, wparam, lparam),
        WM_CHAR => wm_char(hwnd, wparam),
        WM_MOVE => wm_move(hwnd, lparam),
        WM_SIZE => wm_size(hwnd, lparam),
        WM_ENTERSIZEMOVE => wm_enter_size_move(hwnd, wparam, lparam),
        WM_EXITSIZEMOVE => wm_exit_size_move(hwnd, wparam, lparam),
        WM_DPICHANGED => wm_dpi_changed(hwnd, lparam),
        WM_GETDPISCALEDSIZE => wm_get_dpi_scaled_size(hwnd, wparam, lparam),
        WM_ACTIVATE => wm_activate(hwnd, wparam),
        WM_DROPFILES => wm_drop_files(hwnd, wparam),
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
