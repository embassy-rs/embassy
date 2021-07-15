#![macro_use]

use crate::pac;
use crate::peripherals;

pub(crate) unsafe fn configure_dmamux(
    dmamux_regs: pac::dmamux::Dmamux,
    dmamux_ch_num: u8,
    request: u8,
) {
    let ch_mux_regs = dmamux_regs.ccr(dmamux_ch_num as _);
    ch_mux_regs.write(|reg| {
        reg.set_nbreq(0);
        reg.set_dmareq_id(request);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
    });
}

pub(crate) trait MuxChannel {
    const DMAMUX_CH_NUM: u8;
    const DMAMUX_REGS: pac::dmamux::Dmamux;
}

macro_rules! dma_num {
    (DMA1) => {
        0
    };
    (DMA2) => {
        1
    };
}

#[cfg(not(rcc_h7))]
pac::bdma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl MuxChannel for peripherals::$channel_peri {
            const DMAMUX_CH_NUM: u8 = (dma_num!($dma_peri) * 8) + $channel_num;
            const DMAMUX_REGS: pac::dmamux::Dmamux = pac::DMAMUX1;
        }
    };
}

#[cfg(rcc_h7)]
pac::dma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl MuxChannel for peripherals::$channel_peri {
            const DMAMUX_CH_NUM: u8 = (dma_num!($dma_peri) * 8) + $channel_num;
            const DMAMUX_REGS: pac::dmamux::Dmamux = pac::DMAMUX1;
        }
    };
}
#[cfg(rcc_h7)]
pac::bdma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        impl MuxChannel for peripherals::$channel_peri {
            const DMAMUX_CH_NUM: u8 =  $channel_num;
            const DMAMUX_REGS: pac::dmamux::Dmamux = pac::DMAMUX2;
        }
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {}
