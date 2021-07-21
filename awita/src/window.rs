use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::*,
};
use once_cell::sync::OnceCell;
use tokio::sync::{broadcast, oneshot};

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

    #[inline]
    pub async fn build(self) -> anyhow::Result<Window> {
        Window::new(self).await
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

pub(crate) struct WindowState {
    pub mouse_buttons: Vec<MouseButton>,
    pub activated_tx: broadcast::Sender<()>,
    pub inactivated_tx: broadcast::Sender<()>,
    pub mouse_input_tx: broadcast::Sender<event::MouseInput>,
    pub closed_tx: broadcast::Sender<()>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    async fn new(builder: Builder) -> anyhow::Result<Self> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        UiThread::get().send_method(move |ctx| unsafe {
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
            let (activated_tx, _) = broadcast::channel(1);
            let (inactivated_tx, _) = broadcast::channel(1);
            let (mouse_input_tx, _) = broadcast::channel(32);
            let (closed_tx, _) = broadcast::channel(1);
            ctx.insert_window(
                hwnd,
                WindowState {
                    mouse_buttons: Vec::with_capacity(5),
                    activated_tx,
                    inactivated_tx,
                    mouse_input_tx,
                    closed_tx,
                },
            );
            tx.send(Window { hwnd }).ok();
        });
        Ok(rx.await?)
    }

    #[inline]
    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        self.hwnd.0 as _
    }

    async fn on_event<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&WindowState) -> &broadcast::Sender<R> + Send + 'static,
        R: Clone + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::get().send_method(move |ctx| {
            if let Some(state) = ctx.get_window(hwnd) {
                tx.send(f(&state).subscribe()).ok();
            }
        });
        if let Ok(mut event_rx) = rx.await {
            event_rx.recv().await.ok()
        } else {
            None
        }
    }

    #[inline]
    pub async fn on_mouse_input(&self) -> Option<event::MouseInput> {
        self.on_event(|state| &state.mouse_input_tx).await
    }

    #[inline]
    pub async fn on_activated(&self) -> Option<()> {
        self.on_event(|state| &state.activated_tx).await
    }

    #[inline]
    pub async fn on_inactivated(&self) -> Option<()> {
        self.on_event(|state| &state.inactivated_tx).await
    }

    #[inline]
    pub async fn on_closed(&self) -> Option<()> {
        self.on_event(|state| &state.closed_tx).await
    }
}
