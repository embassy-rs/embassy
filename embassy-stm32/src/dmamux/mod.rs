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

use crate::dma_traits::{ReadDma, WriteDma};

pub(crate) fn configure_channel(ch_num: u8, request_num: u8) {}

/*
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
 */

pub(crate) unsafe fn configure_dmamux(
    dmamux_regs: &pac::dmamux::Dmamux,
    dmamux_ch_num: u8,
    request: u8,
) {
    let ch_mux_regs = dmamux_regs.ccr(dmamux_ch_num as _);
    ch_mux_regs.write(|reg| {
        // one request?
        reg.set_nbreq(0);
        reg.set_dmareq_id(request);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
        //reg.set_se(true);
        //reg.set_soie(true);
    });
}

pub(crate) mod sealed {
    use super::*;

    pub trait DmaMux {
        fn regs() -> &'static pac::dmamux::Dmamux;
    }

    pub trait Channel {
        fn num(&self) -> usize;
        fn dma_regs() -> &'static pac::bdma::Dma;
        fn dma_ch_num(&self) -> u8;

        fn dmamux_regs(&self) -> &'static pac::dmamux::Dmamux;
        fn dmamux_ch_num(&self) -> u8;
    }

    pub trait PeripheralChannel<PERI, OP>: Channel {
        fn request(&self) -> u8;
    }
}

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

peripherals! {
    (bdma, DMA1) => {
        dma_channels! {
            ($channel_peri:ident, DMA1, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA1, 0);
            };
        }
    };
    (bdma, DMA2) => {
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
                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }

            };

            (usart, $peri:ident, TX, $request:expr) => {
                impl sealed::PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, M2P> for peripherals::$channel_peri { }

            };

            (uart, $peri:ident, TX, $request:expr) => {
                impl sealed::PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri {
                    fn request(&self) -> u8 {
                        $request
                    }
                }

                impl PeripheralChannel<peripherals::$peri, P2M> for peripherals::$channel_peri { }
            };

            (uart, $peri:ident, RX, $request:expr) => {
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

/// safety: must be called only once
pub(crate) unsafe fn init() {}
