use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    UI::WindowsAndMessaging::*,
};

unsafe fn on_destroy(_hwnd: HWND) -> LRESULT {
    PostQuitMessage(0);
    LRESULT(0)
}

pub(crate) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ret = std::panic::catch_unwind(|| {
        match msg {
            WM_DESTROY => on_destroy(hwnd),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    });
    match ret {
        Ok(ret) => ret,
        Err(e) => {
            context().set_unwind(e);
            LRESULT(0)
        }
    }
}
