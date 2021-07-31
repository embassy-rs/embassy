#[cfg(stm32f401)]
pub(crate) const SYSCLK_MAX: u32 = 84_000_000;

#[cfg(any(stm32f405, stm32f407, stm32f415, stm32f417,))]
pub(crate) const SYSCLK_MAX: u32 = 168_000_000;

#[cfg(any(stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,))]
pub(crate) const SYSCLK_MAX: u32 = 100_000_000;

#[cfg(any(
    stm32f427, stm32f429, stm32f437, stm32f439, stm32f446, stm32f469, stm32f479,
))]
pub(crate) const SYSCLK_MAX: u32 = 180_000_000;

#[cfg(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,))]
pub(crate) const PCLK2_MAX: u32 = SYSCLK_MAX;

#[cfg(not(any(stm32f401, stm32f410, stm32f411, stm32f412, stm32f413, stm32f423,)))]
pub(crate) const PCLK2_MAX: u32 = SYSCLK_MAX / 2;

pub(crate) const PCLK1_MAX: u32 = PCLK2_MAX / 2;
