mod detect;
mod nvidia;
mod platform;
mod types;

pub use detect::detect_hardware;
#[allow(unused_imports)]
pub use types::{GpuDevice, HardwareProfile};
