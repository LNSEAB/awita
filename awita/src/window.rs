use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
};
use once_cell::sync::OnceCell;
use tokio::sync::{mpsc, oneshot};
use async_broadcast::{InactiveReceiver, Sender, broadcast};

pub struct Builder {
    title: String,
    position: ScreenPoint<i32>,
    size: Box<dyn ToPhysical<Value = u32, Output = Size<u32>> + Send>,
    visibility: bool,
    icon: Option<Icon>,
    cursor: Option<Cursor>,
    enable_ime: bool,
    ime_composition_window_visibility: bool,
    ime_candidate_window_visibility: bool,
    accept_drop_files: bool,
}

impl Builder {
    #[inline]
    pub fn new() -> Self {
        Self {
            title: "".into(),
            position: Screen(Point::new(0, 0)),
            size: Box::new(Logical(Size::new(640, 480))),
            visibility: true,
            icon: None,
            cursor: Some(Cursor::Arrow),
            enable_ime: true,
            ime_composition_window_visibility: true,
            ime_candidate_window_visibility: true,
            accept_drop_files: false,
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
    pub fn visible(mut self, visibility: bool) -> Self {
        self.visibility = visibility;
        self
    }

    #[inline]
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    #[inline]
    pub fn cursor(mut self, cursor: Option<Cursor>) -> Self {
        self.cursor = cursor;
        self
    }

    #[inline]
    pub fn ime(mut self, enable: bool) -> Self {
        self.enable_ime = enable;
        self
    }

    #[inline]
    pub fn visible_ime_composition_window(mut self, visibility: bool) -> Self {
        self.ime_composition_window_visibility = visibility;
        self
    }

    #[inline]
    pub fn visible_ime_candidate_window(mut self, visibility: bool) -> Self {
        self.ime_candidate_window_visibility = visibility;
        self
    }

    #[inline]
    pub fn accept_drop_files(mut self, flag: bool) -> Self {
        self.accept_drop_files = flag;
        self
    }

    #[inline]
    pub async fn build(self) -> Result<Window, Error> {
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

pub(crate) struct EventChannel<T> {
    pub tx: Sender<T>,
    pub rx: InactiveReceiver<T>,
}

impl<T> EventChannel<T>
where
    T: Clone
{
    fn new(capacity: usize) -> Self {
        let (mut tx, rx) = broadcast(capacity);
        tx.set_overflow(true);
        Self {
            tx,
            rx: rx.deactivate(),
        }
    }

    #[inline]
    pub fn send(&self, value: T) {
        self.tx.try_broadcast(value).ok();
    }
}

pub(crate) struct WindowState {
    pub cursor: Option<Cursor>,
    pub ime_composition_window_visibility: bool,
    pub ime_candidate_window_visibility: bool,
    pub ime_context: ime::ImmContext,
    pub ime_position: PhysicalPoint<i32>,
    pub draw_channel: EventChannel<()>,
    pub cursor_entered_channel: EventChannel<MouseState>,
    pub cursor_leaved_channel: EventChannel<MouseState>,
    pub cursor_moved_chennel: EventChannel<MouseState>,
    pub mouse_input_channel: EventChannel<event::MouseInput>,
    pub key_input_channel: EventChannel<event::KeyInput>,
    pub char_input_channel: EventChannel<char>,
    pub ime_start_composition_channel: EventChannel<()>,
    pub ime_composition_channel: EventChannel<(ime::Composition, Option<ime::CandidateList>)>,
    pub ime_end_composition_channel: EventChannel<Option<String>>,
    pub moved_channel: EventChannel<ScreenPoint<i32>>,
    pub sizing_channel: EventChannel<PhysicalSize<u32>>,
    pub sized_channel: EventChannel<PhysicalSize<u32>>,
    pub activated_channel: EventChannel<()>,
    pub inactivated_channel: EventChannel<()>,
    pub dpi_changed_channel: EventChannel<u32>,
    pub drop_files_channel: EventChannel<event::DropFiles>,
    pub close_request_channel: Option<mpsc::Sender<event::CloseRequest>>,
    pub closed_channel: EventChannel<()>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Window {
    hwnd: HWND,
}

impl Window {
    async fn new(builder: Builder) -> Result<Self, Error> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        UiThread::post_with_context(move |ctx| unsafe {
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
            if hwnd.is_null() {
                tx.send(Err(windows::HRESULT::from_thread().into())).ok();
                return;
            }
            if let Some(icon) = builder.icon {
                let big = icon.load().unwrap();
                SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG as _), LPARAM(big.0 as _));
                let small = icon.load_small().unwrap();
                SendMessageW(
                    hwnd,
                    WM_SETICON,
                    WPARAM(ICON_SMALL as _),
                    LPARAM(small.0 as _),
                );
            }
            DragAcceptFiles(hwnd, builder.accept_drop_files);
            ShowWindow(hwnd, SW_SHOW);
            ctx.insert_window(
                hwnd,
                WindowState {
                    cursor: builder.cursor,
                    ime_composition_window_visibility: builder.ime_composition_window_visibility,
                    ime_candidate_window_visibility: builder.ime_candidate_window_visibility,
                    ime_context: ime::ImmContext::new(hwnd),
                    ime_position: Physical(Point::new(0, 0)),
                    draw_channel: EventChannel::new(8),
                    cursor_entered_channel: EventChannel::new(8),
                    cursor_leaved_channel: EventChannel::new(8),
                    cursor_moved_chennel: EventChannel::new(128),
                    mouse_input_channel: EventChannel::new(64),
                    key_input_channel: EventChannel::new(256),
                    char_input_channel: EventChannel::new(256),
                    ime_start_composition_channel: EventChannel::new(1),
                    ime_composition_channel: EventChannel::new(1),
                    ime_end_composition_channel: EventChannel::new(1),
                    moved_channel: EventChannel::new(128),
                    sizing_channel: EventChannel::new(128),
                    sized_channel: EventChannel::new(1),
                    activated_channel: EventChannel::new(1),
                    inactivated_channel: EventChannel::new(1),
                    dpi_changed_channel: EventChannel::new(1),
                    drop_files_channel: EventChannel::new(1),
                    close_request_channel: None,
                    closed_channel: EventChannel::new(1),
                },
            );
            if let Some(window) = ctx.get_window(hwnd) {
                if builder.enable_ime {
                    window.ime_context.enable();
                } else {
                    window.ime_context.disable();
                }
            }
            tx.send(Ok(Window { hwnd })).ok();
        });
        rx.await.unwrap()
    }

    #[inline]
    pub async fn dpi(&self) -> Option<u32> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post(move || unsafe {
            tx.send(GetDpiForWindow(hwnd)).ok();
        });
        rx.await.ok()
    }

    #[inline]
    pub fn show(&self) {
        unsafe {
            ShowWindowAsync(self.hwnd, SW_SHOW.0 as _);
        }
    }

    #[inline]
    pub fn hide(&self) {
        unsafe {
            ShowWindowAsync(self.hwnd, SW_HIDE.0 as _);
        }
    }

    #[inline]
    pub fn redraw(&self) {
        let hwnd = self.hwnd.clone();
        UiThread::post(move || unsafe {
            RedrawWindow(hwnd, std::ptr::null(), HRGN::NULL, RDW_INTERNALPAINT);
        });
    }

    #[inline]
    pub fn set_cursor(&self, cursor: Option<Cursor>) {
        let hwnd = self.hwnd.clone();
        UiThread::post_with_context(move |ctx| {
            if let Some(mut window) = ctx.get_window_mut(hwnd) {
                window.cursor = cursor;
            }
        });
    }

    #[inline]
    pub fn set_ime(&self, enable: bool) {
        let hwnd = self.hwnd.clone();
        UiThread::post_with_context(move |ctx| {
            if let Some(window) = ctx.get_window(hwnd) {
                if enable {
                    window.ime_context.enable();
                } else {
                    window.ime_context.disable();
                }
            }
        });
    }

    #[inline]
    pub fn set_ime_position<T>(&self, position: T)
    where
        T: ToPhysical<Output = Point<i32>, Value = i32> + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        UiThread::post_with_context(move |ctx| unsafe {
            if let Some(mut window) = ctx.get_window_mut(hwnd) {
                let dpi = GetDpiForWindow(hwnd) as i32;
                window.ime_position = position.to_physical(dpi);
            }
        });
    }

    #[inline]
    pub fn close_request(&self) {
        unsafe {
            PostMessageW(self.hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }

    #[inline]
    pub fn close(&self) {
        let hwnd = self.hwnd.clone();
        UiThread::post(move || unsafe {
            DestroyWindow(hwnd);
        });
    }

    #[inline]
    pub fn raw_handle(&self) -> *mut std::ffi::c_void {
        self.hwnd.0 as _
    }

    async fn on_event<F, R>(&self, f: F) -> event::Receiver<R>
    where
        F: FnOnce(&WindowState) -> &EventChannel<R> + Send + 'static,
        R: Clone + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post_with_context(move |ctx| {
            if let Some(state) = ctx.get_window(hwnd) {
                tx.send(f(&state).rx.activate_cloned()).ok();
            }
        });
        event::Receiver(rx.await.ok())
    }

    #[inline]
    pub async fn draw_receiver(&self) -> event::Receiver<()> {
        self.on_event(|state| &state.draw_channel).await
    }

    #[inline]
    pub async fn cursor_entered_receiver(&self) -> event::Receiver<MouseState> {
        self.on_event(|state| &state.cursor_entered_channel).await
    }

    #[inline]
    pub async fn cursor_leaved_receiver(&self) -> event::Receiver<MouseState> {
        self.on_event(|state| &state.cursor_leaved_channel).await
    }

    #[inline]
    pub async fn cursor_moved_receiver(&self) -> event::Receiver<MouseState> {
        self.on_event(|state| &state.cursor_moved_chennel).await
    }

    #[inline]
    pub async fn mouse_input_receiver(&self) -> event::Receiver<event::MouseInput> {
        self.on_event(|state| &state.mouse_input_channel).await
    }

    #[inline]
    pub async fn key_input_receiver(&self) -> event::Receiver<event::KeyInput> {
        self.on_event(|state| &state.key_input_channel).await
    }

    #[inline]
    pub async fn char_input_receiver(&self) -> event::Receiver<char> {
        self.on_event(|state| &state.char_input_channel).await
    }

    #[inline]
    pub async fn ime_start_composition_receiver(&self) -> event::Receiver<()> {
        self.on_event(|state| &state.ime_start_composition_channel)
            .await
    }

    #[inline]
    pub async fn ime_composition_receiver(
        &self,
    ) -> event::Receiver<(ime::Composition, Option<ime::CandidateList>)> {
        self.on_event(|state| &state.ime_composition_channel).await
    }

    #[inline]
    pub async fn ime_end_composition_receiver(&self) -> event::Receiver<Option<String>> {
        self.on_event(|state| &state.ime_end_composition_channel)
            .await
    }

    #[inline]
    pub async fn moved_receiver(&self) -> event::Receiver<ScreenPoint<i32>> {
        self.on_event(|state| &state.moved_channel).await
    }

    #[inline]
    pub async fn sizing_receiver(&self) -> event::Receiver<PhysicalSize<u32>> {
        self.on_event(|state| &state.sizing_channel).await
    }

    #[inline]
    pub async fn sized_receiver(&self) -> event::Receiver<PhysicalSize<u32>> {
        self.on_event(|state| &state.sized_channel).await
    }

    #[inline]
    pub async fn activated_receiver(&self) -> event::Receiver<()> {
        self.on_event(|state| &state.activated_channel).await
    }

    #[inline]
    pub async fn inactivated_receiver(&self) -> event::Receiver<()> {
        self.on_event(|state| &state.inactivated_channel).await
    }

    #[inline]
    pub async fn dpi_changed_receiver(&self) -> event::Receiver<u32> {
        self.on_event(|state| &state.dpi_changed_channel).await
    }

    #[inline]
    pub async fn drop_files_receiver(&self) -> event::Receiver<event::DropFiles> {
        self.on_event(|state| &state.drop_files_channel).await
    }

    #[inline]
    pub async fn close_request_receiver(&self) -> event::CloseRequestReceiver {
        let (tx, rx) = mpsc::channel(1);
        let hwnd = self.hwnd.clone();
        UiThread::post_with_context(move |ctx| {
            if let Some(mut window) = ctx.get_window_mut(hwnd) {
                assert!(window.close_request_channel.is_none());
                window.close_request_channel = Some(tx);
            }
        });
        event::CloseRequestReceiver::new(rx)
    }

    #[inline]
    pub async fn closed_receiver(&self) -> event::Receiver<()> {
        self.on_event(|state| &state.closed_channel).await
    }
}
