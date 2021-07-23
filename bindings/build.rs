fn main() {
    windows::build!(
        Windows::Win32::UI::WindowsAndMessaging::*,
        Windows::Win32::UI::KeyboardAndMouseInput::*,
        Windows::Win32::System::LibraryLoader::GetModuleHandleW,
        Windows::Win32::System::Threading::GetCurrentThreadId,
        Windows::Win32::System::Diagnostics::Debug::*,
        Windows::Win32::Foundation::{
            HWND,
            HINSTANCE,
            POINT,
            SIZE,
            RECT,
            WPARAM,
            LPARAM,
            PSTR,
            PWSTR,
            CloseHandle,
        },
        Windows::Win32::Graphics::Gdi::{
            MonitorFromPoint,
            GetMonitorInfoW,
            EnumDisplayMonitors,
            BeginPaint,
            EndPaint,
            GetStockObject,
            RedrawWindow,
            ScreenToClient,
            MONITORINFO,
            PAINTSTRUCT,
        },
        Windows::Win32::UI::HiDpi::*,
        Windows::Win32::UI::Controls::WM_MOUSELEAVE,
    );
}
