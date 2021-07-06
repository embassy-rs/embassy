#![macro_use]
use core::task::Poll;

use atomic_polyfill::{AtomicU8, Ordering};
use embassy::interrupt::{Interrupt, InterruptExt};
use embassy::util::AtomicWaker;
use futures::future::poll_fn;

use crate::interrupt;

use crate::pac::bdma::{regs, vals};

use crate::pac;
use crate::pac::dma_channels;
use crate::pac::dma_requests;
use crate::pac::interrupts;
use crate::pac::peripheral_count;
use crate::pac::peripherals;
use crate::peripherals;

use core::future::Future;

use crate::dma::{ReadDma, WriteDma};

const CH_COUNT: usize = peripheral_count!(DMA) * 8;
const CH_STATUS_NONE: u8 = 0;
const CH_STATUS_COMPLETED: u8 = 1;
const CH_STATUS_ERROR: u8 = 2;

struct State {
    ch_wakers: [AtomicWaker; CH_COUNT],
    ch_status: [AtomicU8; CH_COUNT],
}

impl State {
    const fn new() -> Self {
        const AW: AtomicWaker = AtomicWaker::new();
        const AU: AtomicU8 = AtomicU8::new(CH_STATUS_NONE);
        Self {
            ch_wakers: [AW; CH_COUNT],
            ch_status: [AU; CH_COUNT],
        }
    }
}

static STATE: State = State::new();

#[allow(unused)]
pub(crate) async unsafe fn transfer_p2m(
    ch: &mut impl Channel,
    ch_func: u8,
    src: *const u8,
    dst: &mut [u8],
) {
    unimplemented!()
}

#[allow(unused)]
pub(crate) async unsafe fn transfer_m2p(
    ch: &mut impl Channel,
    ch_func: u8,
    src: &[u8],
    dst: *mut u8,
) {
    defmt::info!(
        "m2p {} func {} {}-{}",
        src.len(),
        ch_func,
        ch.num(),
        ch.dma_ch_num()
    );
    let n = ch.num();

    STATE.ch_status[n].store(CH_STATUS_NONE, Ordering::Release);

    let ch_regs = ch.regs();
    let dmamux_regs = ch.dmamux_regs();
    let ch_mux_regs = dmamux_regs.ccr(ch.dmamux_ch_num() as _);

    ch_mux_regs.write(|reg| {
        // one request?
        reg.set_nbreq(0);
        reg.set_dmareq_id(ch_func);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
        //reg.set_se(true);
        //reg.set_soie(true);
    });

    ch_regs.par().write_value(dst as _);
    ch_regs.mar().write_value(src.as_ptr() as _);
    ch_regs.ndtr().write_value(regs::Ndtr(src.len() as _));

    ch_regs.cr().write(|reg| {
        reg.set_dir(vals::Dir::FROMMEMORY);
        reg.set_msize(vals::Size::BITS8);
        reg.set_minc(vals::Inc::ENABLED);
        reg.set_pinc(vals::Inc::DISABLED);
        reg.set_teie(true);
        reg.set_tcie(true);
        reg.set_en(true);
    });

    let res = poll_fn(|cx| {
        defmt::info!("poll");
        STATE.ch_wakers[n].register(cx.waker());
        match STATE.ch_status[n].load(Ordering::Acquire) {
            CH_STATUS_NONE => Poll::Pending,
            x => Poll::Ready(x),
        }
    })
    .await;

    defmt::info!("cr {:b}", ch_regs.cr().read().0);

    ch_regs.cr().modify(|reg| {
        reg.set_en(false);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(false);
        //reg.set_se(true);
        //reg.set_soie(true);
    });

    // TODO handle error
    assert!(res == CH_STATUS_COMPLETED);
}

pub(crate) mod sealed {
    use super::*;

    pub trait Bdma {
        fn regs() -> &'static pac::bdma::Dma;
        fn num() -> u8;
    }

    pub trait DmaMux {
        fn regs() -> &'static pac::dmamux::Dmamux;
    }

    pub trait Channel {
        fn num(&self) -> usize;
        fn regs(&self) -> pac::bdma::Ch;
        fn dma_regs() -> &'static pac::bdma::Dma;
        fn dma_ch_num(&self) -> u8;

        fn dmamux_regs(&self) -> &'static pac::dmamux::Dmamux;
        fn dmamux_ch_num(&self) -> u8;
    }

    pub trait PeripheralChannel<PERI, OP>: Channel {
        fn request(&self) -> u8;
    }
}

pub trait Bdma: sealed::Bdma {}
pub trait DmaMux: sealed::DmaMux {}
pub trait Channel: sealed::Channel {}
pub trait PeripheralChannel<PERI, OP>: sealed::Channel {}

pub struct P2M;
pub struct M2P;

macro_rules! impl_dma_channel {
    ($channel_peri:ident, $dmamux_peri:ident, $channel_num:expr, $dma_peri: ident, $dma_num:expr) => {
        impl Channel for peripherals::$channel_peri {}
        impl sealed::Channel for peripherals::$channel_peri {
            fn num(&self) -> usize {
                ($dma_num * 8) + $channel_num
            }

            fn regs(&self) -> pac::bdma::Ch {
                Self::dma_regs().ch(self.dma_ch_num() as _)
            }

            fn dma_regs() -> &'static pac::bdma::Dma {
                &crate::pac::$dma_peri
            }

            fn dma_ch_num(&self) -> u8 {
                $channel_num
            }

            fn dmamux_regs(&self) -> &'static pac::dmamux::Dmamux {
                &crate::pac::$dmamux_peri
            }

            fn dmamux_ch_num(&self) -> u8 {
                ($dma_num * 8) + $channel_num
            }
        }

        impl<T> WriteDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, M2P>,
            T: 'static,
        {
            type WriteDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(&'a mut self, buf: &'a [u8], dst: *mut u8) -> Self::WriteDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, M2P>::request(self);
                unsafe { transfer_m2p(self, request, buf, dst) }
            }
        }

        impl<T> ReadDma<T> for peripherals::$channel_peri
        where
            Self: sealed::PeripheralChannel<T, P2M>,
            T: 'static,
        {
            type ReadDmaFuture<'a> = impl Future<Output = ()>;

            fn transfer<'a>(
                &'a mut self,
                src: *const u8,
                buf: &'a mut [u8],
            ) -> Self::ReadDmaFuture<'a>
            where
                T: 'a,
            {
                let request = sealed::PeripheralChannel::<T, P2M>::request(self);
                unsafe { transfer_p2m(self, request, src, buf) }
            }
        }
    };
}

macro_rules! impl_dmamux {
    ($peri:ident) => {
        impl sealed::DmaMux for peripherals::$peri {
            fn regs() -> &'static pac::dmamux::Dmamux {
                &pac::$peri
            }
        }
        impl DmaMux for peripherals::$peri {}
    };
}

macro_rules! impl_bdma {
    ($peri:ident, $dma_num:expr) => {
        impl sealed::Bdma for peripherals::$peri {
            fn num() -> u8 {
                $dma_num
            }

            fn regs() -> &'static pac::bdma::Dma {
                &pac::$peri
            }
        }

        impl Bdma for peripherals::$peri {}
    };
}

peripherals! {
    (bdma, DMA1) => {
        impl_bdma!(DMA1, 0);
        dma_channels! {
            ($channel_peri:ident, DMA1, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA1, 0);
            };
        }
    };
    (bdma, DMA2) => {
        impl_bdma!(DMA2, 1);
        dma_channels! {
            ($channel_peri:ident, DMA2, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA2, 1);
            };
        }
    };
    (dmamux, DMAMUX1) => {
        impl_dmamux!(DMAMUX1);
    };
}

macro_rules! impl_usart_dma_requests {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        dma_requests! {
            // TODO: DRY this up.
            (usart, $peri:ident, RX, $request:expr) => {
                impl usart::RxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::RxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }

            };

            (usart, $peri:ident, TX, $request:expr) => {
                impl usart::TxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::TxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }

            };

            (uart, $peri:ident, TX, $request:expr) => {
                impl usart::RxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::RxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }
            };

            (uart, $peri:ident, RX, $request:expr) => {
                impl usart::TxDma<peripherals::$peri> for peripherals::$channel_peri { }
                impl usart::sealed::TxDma<peripherals::$peri> for peripherals::$channel_peri { }

                impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }
            };
        }

    };
}

#[cfg(usart)]
use crate::usart;

dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl_usart_dma_requests!($channel_peri, $dma_peri, $channel_num);
    };
}

unsafe fn on_irq() {
    defmt::info!("irq fire");
    peripherals! {
        //(bdma, $dma:ident) => {
        (bdma, DMA1) => {
            defmt::info!("---> dma DMA1");
            //for isrn in 0..2 {
                //let isr = pac::$dma.isr(isrn).read();
                let isr = pac::DMA1.isr().read();
                pac::DMA1.ifcr().write_value(isr);
                let dman = <peripherals::DMA1 as sealed::Bdma>::num() as usize;

                for chn in 0..8 {
                    let n = dman * 8 + chn;
                    defmt::info!("n={}", n);
                    if isr.teif(chn) {
                        defmt::info!("transfer error");
                        STATE.ch_status[n].store(CH_STATUS_ERROR, Ordering::Release);
                        STATE.ch_wakers[n].wake();
                    } else if isr.tcif(chn) {
                        defmt::info!("transfer complete");
                        STATE.ch_status[n].store(CH_STATUS_COMPLETED, Ordering::Release);
                        STATE.ch_wakers[n].wake();
                    } else if isr.htif(chn) {
                        defmt::info!("half transfer");
                    } else if isr.gif(chn) {
                        defmt::info!("half transfer");
                    }
                }
            //}
        };

        (bdma, DMA2) => {
            defmt::info!("---> dma DMA2");
            //for isrn in 0..2 {
                //let isr = pac::$dma.isr(isrn).read();
                let isr = pac::DMA2.isr().read();
                pac::DMA2.ifcr().write_value(isr);
                let dman = <peripherals::DMA2 as sealed::Bdma>::num() as usize;

                for chn in 0..8 {
                    let n = dman * 8 + chn;
                    defmt::info!("n={}", n);
                    if isr.teif(chn) {
                        defmt::info!("transfer error");
                        STATE.ch_status[n].store(CH_STATUS_ERROR, Ordering::Release);
                        STATE.ch_wakers[n].wake();
                    } else if isr.tcif(chn) {
                        defmt::info!("transfer complete");
                        STATE.ch_status[n].store(CH_STATUS_COMPLETED, Ordering::Release);
                        STATE.ch_wakers[n].wake();
                    } else if isr.htif(chn) {
                        defmt::info!("half transfer");
                    } else if isr.gif(chn) {
                        defmt::info!("half transfer");
                    }
                }
            //}
        };

    }
    defmt::info!("irq fire complete");
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    interrupts! {
        (DMA, $irq:ident) => {
            defmt::info!("enable irq {}", stringify!($irq));
            interrupt::$irq::steal().enable();
        };
    }
}

interrupts! {
    (DMA, $irq:ident) => {
        #[crate::interrupt]
        unsafe fn $irq () {
            defmt::info!("irq firing {}", stringify!($irq));
            on_irq()
        }
    };
}
