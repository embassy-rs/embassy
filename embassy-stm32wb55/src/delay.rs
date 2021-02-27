use core::future::Future;

use crate::hal::hal::timer::CountDown;
use crate::hal::time::U32Ext;
use crate::interrupt;

use async_embedded_traits::delay::{AsyncDelayMs, AsyncDelayUs};
use embassy::util::InterruptFuture;
use stm32wb_hal::lptim::{lptim1::LpTimer as LpTimer1, lptim2::LpTimer as LpTimer2, OneShot};

macro_rules! impl_async_delay_lptim {
    ($LpTimer:ty, $lptim_int:path, $lptim:ident) => {
        pub mod $lptim {
            use super::*;

            pub struct LptimDelay {
                lptim: $LpTimer,
                lptim_int: $lptim_int,
            }

            impl LptimDelay {
                pub fn new(lptim: $LpTimer, lptim_int: $lptim_int) -> Self {
                    Self { lptim, lptim_int }
                }

                fn set_irq(&mut self, enable: bool) {
                    if enable {
                        self.lptim.enable_interrupts(crate::hal::lptim::Interrupts {
                            autoreload_match: true,
                            ..Default::default()
                        });
                    } else {
                        self.lptim.clear_interrupts(crate::hal::lptim::Interrupts {
                            autoreload_match: true,
                            ..Default::default()
                        });
                    }
                }
            }

            impl AsyncDelayUs<u8> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_us(&mut self, us: u8) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start((us as u32).us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }

            impl AsyncDelayUs<u16> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_us(&mut self, us: u16) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start((us as u32).us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }

            impl AsyncDelayUs<u32> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_us(&mut self, us: u32) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start(us.us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }

            impl AsyncDelayMs<u8> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_ms(&mut self, ms: u8) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start((ms as u32 * 1000).us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }

            impl AsyncDelayMs<u16> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_ms(&mut self, ms: u16) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start((ms as u32 * 1000).us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }

            impl AsyncDelayMs<u32> for LptimDelay {
                type DelayFuture<'a> = impl Future<Output = ()>;

                fn async_delay_ms(&mut self, ms: u32) -> Self::DelayFuture<'_> {
                    async move {
                        self.set_irq(true);
                        self.lptim.start((ms * 1000).us());
                        InterruptFuture::new(&mut self.lptim_int).await;
                        self.set_irq(false);
                    }
                }
            }
        }
    };
}

impl_async_delay_lptim!(LpTimer1<OneShot>, interrupt::LPTIM1, lptim1);
impl_async_delay_lptim!(LpTimer2<OneShot>, interrupt::LPTIM2, lptim2);
