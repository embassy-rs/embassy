#![macro_use]

use crate::pac;
use crate::pac::bdma_channels;
use crate::pac::dma_requests;
use crate::pac::peripherals;
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

pub(crate) mod sealed {
    use super::*;

    pub trait Channel {
        const DMAMUX_CH_NUM: u8;
        const DMAMUX_REGS: pac::dmamux::Dmamux;
    }

    pub trait PeripheralChannel<PERI, OP>: Channel {
        const REQUEST: u8;
    }
}

pub trait Channel: sealed::Channel {}
pub trait PeripheralChannel<PERI, OP>: sealed::Channel {}

pub struct P2M;
pub struct M2P;

macro_rules! dma_num {
    (DMA1) => {
        0
    };
    (DMA2) => {
        1
    };
    (BDMA) => {
        0
    };
}

macro_rules! dmamux_peri {
    (DMA1) => {
        crate::pac::DMAMUX1
    };
    (DMA2) => {
        crate::pac::DMAMUX1
    };
    (BDMA) => {
        crate::pac::DMAMUX1
    };
}

#[allow(unused)]
macro_rules! impl_dma_channel {
    ($channel_peri:ident, $channel_num:expr, $dma_peri: ident) => {
        impl Channel for peripherals::$channel_peri {}
        impl sealed::Channel for peripherals::$channel_peri {
            const DMAMUX_CH_NUM: u8 = (dma_num!($dma_peri) * 8) + $channel_num;
            const DMAMUX_REGS: pac::dmamux::Dmamux = dmamux_peri!($dma_peri);
        }
    };
}

peripherals! {
    (bdma, $peri:ident) => {
        bdma_channels! {
            ($channel_peri:ident, $peri, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, $channel_num, $peri);
            };
        }
    };
}

#[allow(unused)]
macro_rules! impl_peripheral_channel {
    ($channel_peri:ident, $direction:ident, $peri:ident, $request:expr) => {
        impl sealed::PeripheralChannel<peripherals::$peri, $direction>
            for peripherals::$channel_peri
        {
            const REQUEST: u8 = $request;
        }

        impl PeripheralChannel<peripherals::$peri, $direction> for peripherals::$channel_peri {}
    };
}

#[allow(unused)]
macro_rules! impl_usart_dma_requests {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        dma_requests! {
            (usart, $peri:ident, RX, $request:expr) => {
                impl_peripheral_channel!($channel_peri, P2M, $peri, $request);
            };

            (usart, $peri:ident, TX, $request:expr) => {
                impl_peripheral_channel!($channel_peri, M2P, $peri, $request);
            };

            (uart, $peri:ident, RX, $request:expr) => {
                impl_peripheral_channel!($channel_peri, P2M, $peri, $request);
            };

            (uart, $peri:ident, TX, $request:expr) => {
                impl_peripheral_channel!($channel_peri, M2P, $peri, $request);
            };
        }
    };
}

#[allow(unused)]
#[cfg(usart)]
use crate::usart;

bdma_channels! {
    ($channel_peri:ident, $dma_peri:ident, $channel_num:expr) => {
        #[cfg(usart)]
        impl_usart_dma_requests!($channel_peri, $dma_peri, $channel_num);
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {}
