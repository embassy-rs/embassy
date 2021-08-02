
#[cfg(feature="stm32f401")]
mod stm32f401;
#[cfg(feature="stm32f401")]
pub use stm32f401::*;

#[cfg(feature="stm32f429")]
mod stm32f429;
#[cfg(feature="stm32f429")]
pub use stm32f429::*;
