use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};
use once_cell::sync::OnceCell;

pub struct Builder {
    pub(crate) title: String,
    pub(crate) position: ScreenPoint<i32>,
    pub(crate) size: Box<dyn ToPhysical<Value = u32, Output = Size<u32>> + Send>,
}

impl Builder {
    #[inline]
    pub fn new() -> Self {
        Self {
            title: "".into(),
            position: Screen(Point::new(0, 0)),
            size: Box::new(Logical(Size::new(640, 480))),
        }
    }

    #[inline]
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().into();
        self
    }

    #[inline]
    pub fn position(mut self, position: ScreenPoint<i32>) -> Self {
        self.position = position;
        self
    }

    #[inline]
    pub fn size<S>(mut self, size: S) -> Self
    where
        S: ToPhysical<Value = u32, Output = Size<u32>> + Send + 'static,
    {
        self.size = Box::new(size);
        self
    }

    pub async fn build(self) -> anyhow::Result<Window> {
        let ui_thread = UiThread::get();
        let (tx, rx) = tokio::sync::oneshot::channel();
        ui_thread.send_method(move || {
            tx.send(Window::new(self)).ok();
        });
        Ok(rx.await.unwrap()?)
    }
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "window::Builder({})", self.title)
    }
}

fn window_class() -> &'static Vec<u16> {
    static CLASS_NAME: OnceCell<Vec<u16>> = OnceCell::new();
    CLASS_NAME.get_or_init(|| unsafe {
        let class_name = "awita_window_class"
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(procedure::window_proc),
            hCursor: LoadCursorW(None, IDC_ARROW),
            hInstance: GetModuleHandleW(None),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpszClassName: PWSTR(class_name.as_ptr() as _),
            ..Default::default()
        };
        if RegisterClassExW(&wc) == 0 {
            panic!("RegisterClassEx failed");
        }
        class_name
    })
}

#[derive(Debug)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    fn new(builder: Builder) -> anyhow::Result<Self> {
        unsafe {
            let title = builder
                .title
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let size = builder.size.to_physical(DEFAULT_DPI as _);
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                PWSTR(window_class().as_ptr() as _),
                PWSTR(title.as_ptr() as _),
                WS_OVERLAPPEDWINDOW,
                builder.position.x,
                builder.position.y,
                size.width as _,
                size.height as _,
                None,
                None,
                GetModuleHandleW(None),
                std::ptr::null_mut(),
            );
            ShowWindow(hwnd, SW_SHOW);
            Ok(Window { hwnd })
        }
    }
}
