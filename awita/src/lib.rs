mod device;
pub mod event;
pub mod geometry;
mod procedure;
mod ui_thread;
mod utility;
pub mod window;

pub use device::*;
pub use geometry::*;
pub use window::Window;

use ui_thread::*;

#[inline]
pub fn join() -> Result<(), Box<dyn std::any::Any + Send>> {
    UiThread::get().join()
}
