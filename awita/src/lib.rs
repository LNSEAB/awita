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
pub async fn finished() {
    ui_thread::UiThread::finished().await;
}
