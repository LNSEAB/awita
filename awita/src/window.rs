use super::*;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{HiDpi::*, Shell::*, WindowsAndMessaging::*},
};
use once_cell::sync::OnceCell;
use tokio::sync::{mpsc, oneshot};

pub trait StyleObject {
    fn value(&self) -> u32;
    fn ex(&self) -> u32;
}

#[derive(Clone, Copy, Debug)]
pub struct BorderlessStyle;

impl StyleObject for BorderlessStyle {
    fn value(&self) -> u32 {
        WS_POPUP.0
    }

    fn ex(&self) -> u32 {
        0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Style {
    value: u32,
    ex: u32,
}

impl Style {
    #[inline]
    pub const fn new() -> Self {
        Self {
            value: WS_OVERLAPPEDWINDOW.0,
            ex: 0,
        }
    }

    #[inline]
    pub const fn dialog() -> Self {
        Self {
            value: WS_OVERLAPPED.0 | WS_CAPTION.0 | WS_SYSMENU.0,
            ex: 0,
        }
    }

    #[inline]
    pub const fn borderless() -> BorderlessStyle {
        BorderlessStyle
    }

    #[inline]
    pub const fn resizable(mut self, resizable: bool) -> Self {
        if resizable {
            self.value |= WS_THICKFRAME.0;
        } else {
            self.value &= !WS_THICKFRAME.0;
        }
        self
    }

    #[inline]
    pub const fn has_minimize_box(mut self, flag: bool) -> Self {
        if flag {
            self.value |= WS_MINIMIZEBOX.0;
        } else {
            self.value &= !WS_MINIMIZEBOX.0;
        }
        self
    }

    #[inline]
    pub const fn has_maximize_box(mut self, flag: bool) -> Self {
        if flag {
            self.value |= WS_MAXIMIZEBOX.0;
        } else {
            self.value &= !WS_MAXIMIZEBOX.0;
        }
        self
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

impl StyleObject for Style {
    fn value(&self) -> u32 {
        self.value
    }

    fn ex(&self) -> u32 {
        self.ex
    }
}

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
    style: Style,
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
            style: Style::new(),
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
    pub fn enable_ime(mut self, enable: bool) -> Self {
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
    pub fn style(mut self, object: impl StyleObject) -> Self {
        self.style = Style {
            value: object.value(),
            ex: object.ex(),
        };
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

pub(crate) struct WindowState {
    pub cursor: Option<Cursor>,
    pub ime_composition_window_visibility: bool,
    pub ime_candidate_window_visibility: bool,
    pub ime_context: ime::ImmContext,
    pub ime_position: PhysicalPoint<i32>,
    pub draw_channel: event::Channel<()>,
    pub cursor_entered_channel: event::Channel<MouseState>,
    pub cursor_leaved_channel: event::Channel<MouseState>,
    pub cursor_moved_chennel: event::Channel<MouseState>,
    pub mouse_input_channel: event::Channel<event::MouseInput>,
    pub key_input_channel: event::Channel<event::KeyInput>,
    pub char_input_channel: event::Channel<char>,
    pub ime_start_composition_channel: event::Channel<()>,
    pub ime_composition_channel: event::Channel<(ime::Composition, Option<ime::CandidateList>)>,
    pub ime_end_composition_channel: event::Channel<Option<String>>,
    pub moved_channel: event::Channel<ScreenPoint<i32>>,
    pub sizing_channel: event::Channel<PhysicalSize<u32>>,
    pub sized_channel: event::Channel<PhysicalSize<u32>>,
    pub activated_channel: event::Channel<()>,
    pub inactivated_channel: event::Channel<()>,
    pub dpi_changed_channel: event::Channel<u32>,
    pub drop_files_channel: event::Channel<event::DropFiles>,
    pub close_request_channel: Option<mpsc::Sender<event::CloseRequest>>,
    pub closed_channel: event::Channel<()>,
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
                WINDOW_EX_STYLE(builder.style.ex),
                PWSTR(window_class().as_ptr() as _),
                PWSTR(title.as_ptr() as _),
                WINDOW_STYLE(builder.style.value),
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
                    draw_channel: event::Channel::new(8),
                    cursor_entered_channel: event::Channel::new(8),
                    cursor_leaved_channel: event::Channel::new(8),
                    cursor_moved_chennel: event::Channel::new(128),
                    mouse_input_channel: event::Channel::new(64),
                    key_input_channel: event::Channel::new(256),
                    char_input_channel: event::Channel::new(256),
                    ime_start_composition_channel: event::Channel::new(1),
                    ime_composition_channel: event::Channel::new(1),
                    ime_end_composition_channel: event::Channel::new(1),
                    moved_channel: event::Channel::new(128),
                    sizing_channel: event::Channel::new(128),
                    sized_channel: event::Channel::new(1),
                    activated_channel: event::Channel::new(1),
                    inactivated_channel: event::Channel::new(1),
                    dpi_changed_channel: event::Channel::new(1),
                    drop_files_channel: event::Channel::new(1),
                    close_request_channel: None,
                    closed_channel: event::Channel::new(1),
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
        rx.await?
    }

    #[inline]
    pub async fn title(&self) -> Result<String, Error> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post(move || unsafe {
            let len = GetWindowTextLengthW(hwnd) as usize;
            if len == 0 {
                tx.send(String::new()).ok();
                return;
            }
            let len = len + 1;
            let mut buf: Vec<u16> = Vec::with_capacity(len);
            buf.set_len(len);
            GetWindowTextW(hwnd, PWSTR(buf.as_mut_ptr()), len as _);
            buf.pop();
            tx.send(String::from_utf16_lossy(&buf)).ok();
        });
        Ok(rx.await?)
    }

    #[inline]
    pub async fn set_title(&self, text: impl AsRef<str>) {
        let hwnd = self.hwnd.clone();
        let mut text = text
            .as_ref()
            .encode_utf16()
            .chain(Some(0))
            .collect::<Vec<_>>();
        UiThread::post(move || unsafe {
            SetWindowTextW(hwnd, PWSTR(text.as_mut_ptr()));
        });
    }

    #[inline]
    pub async fn position(&self) -> Result<Screen<Point<i32>>, Error> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post(move || unsafe {
            let mut rc = RECT::default();
            GetWindowRect(hwnd, &mut rc);
            tx.send(Screen(Point::new(rc.left, rc.top))).ok();
        });
        Ok(rx.await?)
    }

    #[inline]
    pub fn set_position<T>(&self, position: T)
    where
        T: ToPhysical<Output = Point<i32>, Value = i32> + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        UiThread::post(move || unsafe {
            let dpi = GetDpiForWindow(hwnd) as i32;
            let position = position.to_physical(dpi);
            SetWindowPos(
                hwnd,
                HWND::NULL,
                position.x,
                position.y,
                0,
                0,
                SWP_NOZORDER | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        });
    }

    #[inline]
    pub async fn inner_size(&self) -> Result<Physical<Size<u32>>, Error> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post(move || unsafe {
            let mut rc = RECT::default();
            GetClientRect(hwnd, &mut rc);
            tx.send(Physical(Size::new(rc.right as _, rc.bottom as _)))
                .ok();
        });
        Ok(rx.await?)
    }

    #[inline]
    pub fn set_inner_size<T>(&self, size: T)
    where
        T: ToPhysical<Output = Size<u32>, Value = u32> + Send + 'static,
    {
        let hwnd = self.hwnd.clone();
        UiThread::post(move || unsafe {
            let dpi = GetDpiForWindow(hwnd);
            let size = size.to_physical(dpi);
            SetWindowPos(
                hwnd,
                HWND::NULL,
                0,
                0,
                size.width as _,
                size.height as _,
                SWP_NOZORDER | SWP_NOMOVE | SWP_NOACTIVATE,
            );
        });
    }

    #[inline]
    pub async fn dpi(&self) -> Result<u32, Error> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post(move || unsafe {
            tx.send(GetDpiForWindow(hwnd)).ok();
        });
        Ok(rx.await?)
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
    pub async fn is_enabled_ime(&self) -> Result<bool, Error> {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post_with_context(move |ctx| {
            let enabled = if let Some(window) = ctx.get_window(hwnd) {
                Ok(window.ime_context.is_enabled())
            } else {
                Err(Error::Closed)
            };
            tx.send(enabled).ok();
        });
        rx.await?
    }

    #[inline]
    pub fn set_enable_ime(&self, enable: bool) {
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
    pub async fn is_closed(&self) -> bool {
        let hwnd = self.hwnd.clone();
        let (tx, rx) = oneshot::channel();
        UiThread::post_with_context(move |ctx| {
            tx.send(ctx.get_window(hwnd).is_none()).ok();
        });
        rx.await.unwrap_or(true)
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
        F: FnOnce(&WindowState) -> &event::Channel<R> + Send + 'static,
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
