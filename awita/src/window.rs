use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, WindowsAndMessaging::*},
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

fn get_dpi_from_point(pt: ScreenPoint<i32>) -> u32 {
    unsafe {
        let mut dpi_x = 0;
        let mut _dpi_y = 0;
        GetDpiForMonitor(
            MonitorFromPoint(POINT { x: pt.x, y: pt.y }, MONITOR_DEFAULTTONEAREST),
            MDT_DEFAULT,
            &mut dpi_x,
            &mut _dpi_y,
        )
        .unwrap();
        dpi_x
    }
}

pub(crate) struct WindowState {
    pub mouse_input_channel: broadcast::Sender<event::MouseInput>,
    pub activated_channel: broadcast::Sender<()>,
    pub inactivated_channel: broadcast::Sender<()>,
    pub dpi_changed_channel: broadcast::Sender<u32>,
    pub closed_channel: broadcast::Sender<()>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    async fn new(builder: Builder) -> anyhow::Result<Self> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        UiThread::get().send_method(move |ctx| unsafe {
            let dpi = get_dpi_from_point(builder.position);
            let title = builder
                .title
                .encode_utf16()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let size = builder.size.to_physical(dpi);
            let size =
                utility::adjust_window_size(size, WS_OVERLAPPEDWINDOW, WINDOW_EX_STYLE(0), dpi);
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
            ctx.insert_window(
                hwnd,
                WindowState {
                    mouse_input_channel: broadcast::channel(64).0,
                    activated_channel: broadcast::channel(1).0,
                    inactivated_channel: broadcast::channel(1).0,
                    dpi_changed_channel: broadcast::channel(1).0,
                    closed_channel: broadcast::channel(1).0,
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

    async fn on_event<F, R>(&self, f: F) -> Option<broadcast::Receiver<R>>
    where
        F: FnOnce(&WindowState) -> &broadcast::Sender<R> + Send + 'static,
        R: Clone + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::get().send_method(move |ctx| {
            if let Some(state) = ctx.get_window(hwnd) {
                tx.send(f(&state).subscribe()).unwrap();
            }
        });
        rx.await.ok()
    }

    #[inline]
    pub async fn mouse_input_receiver(&self) -> event::Receiver<event::MouseInput> {
        event::Receiver(self.on_event(|state| &state.mouse_input_channel).await)
    }

    #[inline]
    pub async fn activated_receiver(&self) -> event::Receiver<()> {
        event::Receiver(self.on_event(|state| &state.activated_channel).await)
    }

    #[inline]
    pub async fn inactivated_receiver(&self) -> event::Receiver<()> {
        event::Receiver(self.on_event(|state| &state.inactivated_channel).await)
    }

    #[inline]
    pub async fn dpi_changed_receiver(&self) -> event::Receiver<u32> {
        event::Receiver(self.on_event(|state| &state.dpi_changed_channel).await)
    }

    #[inline]
    pub async fn closed_receiver(&self) -> event::Receiver<()> {
        event::Receiver(self.on_event(|state| &state.closed_channel).await)
    }
}
