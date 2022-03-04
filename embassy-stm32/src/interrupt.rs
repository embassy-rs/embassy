pub use bare_metal::Mutex;
pub use critical_section::CriticalSection;
pub use embassy::interrupt::{take, Interrupt};
pub use embassy_hal_common::interrupt::Priority4 as Priority;

pub use crate::_generated::interrupt::*;
