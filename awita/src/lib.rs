mod device;
pub mod event;
pub mod geometry;
mod procedure;
mod resource;
mod ui_thread;
mod utility;
pub mod window;
mod error;

pub use device::*;
pub use geometry::*;
pub use resource::*;
pub use window::Window;
pub use error::*;

pub use ui_thread::UiThread;
use ui_thread::{Context, CONTEXT};

#[inline]
pub async fn finished() {
    ui_thread::UiThread::finished().await;
}
