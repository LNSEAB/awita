use crate::window::WindowState;
use awita_windows_bindings::Windows::Win32::{
    Foundation::*,
    System::Threading::*,
    UI::{HiDpi::*, WindowsAndMessaging::*},
};
use once_cell::sync::OnceCell;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::RwLock;
use tokio::sync::mpsc;

const WM_AWITA_METHOD: u32 = WM_APP + 1;

pub(crate) struct UiThread {
    thread: RwLock<Option<std::thread::JoinHandle<()>>>,
    thread_id: u32,
    method_tx: mpsc::UnboundedSender<Box<dyn FnOnce(&Context) + Send>>,
}

impl UiThread {
    pub fn get() -> &'static UiThread {
        static UI_THREAD: OnceCell<UiThread> = OnceCell::new();
        UI_THREAD.get_or_init(|| run())
    }

    pub fn send_method(&self, method: impl FnOnce(&Context) + Send + 'static) {
        unsafe {
            PostThreadMessageW(self.thread_id, WM_AWITA_METHOD, WPARAM(0), LPARAM(0));
        }
        self.method_tx.send(Box::new(method)).ok();
    }

    pub fn join(&self) -> Result<(), Box<dyn std::any::Any + Send>> {
        let thread = {
            let mut thread = self.thread.write().unwrap();
            thread.take().unwrap()
        };
        thread.join()
    }
}

pub(crate) struct Context {
    pub(crate) runtime: tokio::runtime::Runtime,
    method_rx: RefCell<mpsc::UnboundedReceiver<Box<dyn FnOnce(&Context) + Send>>>,
    window_map: RefCell<HashMap<isize, WindowState>>,
    unwind: RefCell<Option<Box<dyn std::any::Any + Send>>>,
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
    let thread = std::thread::spawn(move || unsafe {
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
                std::panic::resume_unwind(e);
            }
        }
    });
    UiThread {
        thread: RwLock::new(Some(thread)),
        thread_id: id_rx.recv().unwrap(),
        method_tx: tx,
    }
}
