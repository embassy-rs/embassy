pub use bare_metal::Mutex;
pub use critical_section::CriticalSection;
pub use embassy::interrupt::{take, Interrupt};
pub use embassy_extras::interrupt::Priority4 as Priority;

use crate::pac::Interrupt as InterruptEnum;
use embassy::interrupt::declare;

crate::pac::interrupts!(
    ($name:ident) => {
        declare!($name);
    };
);
