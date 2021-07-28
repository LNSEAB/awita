use crate::window::WindowState;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    System::{Com::*, Threading::*},
    UI::{HiDpi::*, WindowsAndMessaging::*},
};
use once_cell::sync::OnceCell;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::{mpsc, watch, oneshot, Mutex};

const WM_AWITA_METHOD: u32 = WM_APP + 1;

static UI_THREAD: OnceCell<UiThread> = OnceCell::new();

pub struct UiThread {
    thread_id: u32,
    method_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&Context) + Send>>,
    finish_rx: watch::Receiver<bool>,
    unwind_rx: Mutex<Option<oneshot::Receiver<Option<Box<dyn std::any::Any + Send>>>>>,
}

impl UiThread {
    fn get() -> &'static UiThread {
        UI_THREAD.get_or_init(|| run())
    }

    pub fn post(f: impl FnOnce() + Send + 'static) {
        let th = Self::get();
        unsafe {
            PostThreadMessageW(th.thread_id, WM_AWITA_METHOD, WPARAM(0), LPARAM(0));
        }
        th.method_tx.send(Box::new(|_| f())).ok();
    }

    pub(crate) fn post_with_context(f: impl FnOnce(&Context) + Send + 'static) {
        let th = Self::get();
        unsafe {
            PostThreadMessageW(th.thread_id, WM_AWITA_METHOD, WPARAM(0), LPARAM(0));
        }
        th.method_tx.send(Box::new(f)).ok();
    }

    pub async fn finished() -> bool {
        let mut finish_rx = Self::get().finish_rx.clone();
        finish_rx.changed().await.unwrap();
        let value = *finish_rx.borrow();
        value
    }

    pub async fn resume_unwind() {
        let mut rx = Self::get().unwind_rx.lock().await;
        let rx = rx.take().unwrap();
        if let Some(e) = rx.await.unwrap() {
            std::panic::resume_unwind(e);
        }
    }
}

pub(crate) struct Context {
    pub(crate) runtime: tokio::runtime::Runtime,
    method_rx: RefCell<mpsc::UnboundedReceiver<Box<dyn FnOnce(&Context) + Send>>>,
    window_map: RefCell<HashMap<isize, WindowState>>,
    unwind: RefCell<Option<Box<dyn std::any::Any + Send>>>,
    pub(crate) resizing: Cell<bool>,
    pub(crate) entered_cursor_window: Cell<Option<HWND>>,
}

impl Context {
    fn new(method_rx: mpsc::UnboundedReceiver<Box<dyn FnOnce(&Context) + Send>>) -> Rc<Self> {
        Rc::new(Self {
            runtime: tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap(),
            method_rx: RefCell::new(method_rx),
            window_map: RefCell::new(HashMap::new()),
            unwind: RefCell::new(None),
            resizing: Cell::new(false),
            entered_cursor_window: Cell::new(None),
        })
    }

    fn process_method(self: Rc<Self>) {
        let local = tokio::task::LocalSet::new();
        let ctx = self.clone();
        let f = async {
            let mut rx = ctx.method_rx.borrow_mut();
            if let Some(method) = rx.recv().await {
                method(self.as_ref());
            }
        };
        local.block_on(&self.runtime, f);
    }

    pub fn insert_window(&self, hwnd: HWND, state: WindowState) {
        self.window_map.borrow_mut().insert(hwnd.0, state);
    }

    pub fn remove_window(&self, hwnd: HWND) {
        self.window_map.borrow_mut().remove(&hwnd.0);
    }

    pub fn window_map_is_empty(&self) -> bool {
        self.window_map.borrow().is_empty()
    }

    pub fn get_window(&self, hwnd: HWND) -> Option<Ref<WindowState>> {
        let window_map = self.window_map.borrow();
        if window_map.get(&hwnd.0).is_none() {
            None
        } else {
            Some(Ref::map(window_map, |m| m.get(&hwnd.0).unwrap()))
        }
    }

    pub fn get_window_mut(&self, hwnd: HWND) -> Option<RefMut<WindowState>> {
        let window_map = self.window_map.borrow_mut();
        if window_map.get(&hwnd.0).is_none() {
            None
        } else {
            Some(RefMut::map(window_map, |m| m.get_mut(&hwnd.0).unwrap()))
        }
    }

    pub fn set_unwind(&self, e: Box<dyn std::any::Any + Send>) {
        *self.unwind.borrow_mut() = Some(e);
    }
}

thread_local! {
    pub(crate) static CONTEXT: RefCell<Option<Rc<Context>>> = RefCell::new(None);
}

fn context() -> Rc<Context> {
    CONTEXT.with(|ctx| ctx.borrow().as_ref().unwrap().clone())
}

fn run() -> UiThread {
    let (tx, rx) = mpsc::unbounded_channel();
    let (id_tx, id_rx) = std::sync::mpsc::channel();
    let (finish_tx, finish_rx) = watch::channel(false);
    let (unwind_tx, unwind_rx) = oneshot::channel();
    std::thread::spawn(move || unsafe {
        CoInitialize(std::ptr::null_mut()).unwrap();
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        IsGUIThread(true);
        CONTEXT.with(|ctx| {
            *ctx.borrow_mut() = Some(Context::new(rx));
        });
        id_tx.send(GetCurrentThreadId()).unwrap();
        let mut msg = MSG::default();
        loop {
            let ret = GetMessageW(&mut msg, HWND::NULL, 0, 0).0;
            if ret == 0 || ret == -1 {
                break;
            }
            if msg.message == WM_AWITA_METHOD {
                context().process_method();
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if let Some(e) = context().unwind.take() {
                unwind_tx.send(Some(e)).ok();
                finish_tx.send(false).ok();
                return;
            }
        }
        unwind_tx.send(None).ok();
        finish_tx.send(true).ok();
    });
    UiThread {
        thread_id: id_rx.recv().unwrap(),
        method_tx: tx,
        finish_rx,
        unwind_rx: Mutex::new(Some(unwind_rx)),
    }
}
