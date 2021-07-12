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

    pub trait DmaMux {
        fn regs() -> pac::dmamux::Dmamux;
    }

    pub trait Channel {
        fn dmamux_regs(&self) -> pac::dmamux::Dmamux;
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

#[allow(unused)]
macro_rules! impl_dma_channel {
    ($channel_peri:ident, $dmamux_peri:ident, $channel_num:expr, $dma_peri: ident, $dma_num:expr) => {
        impl Channel for peripherals::$channel_peri {}
        impl sealed::Channel for peripherals::$channel_peri {
            fn dmamux_regs(&self) -> pac::dmamux::Dmamux {
                crate::pac::$dmamux_peri
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
            fn regs() -> pac::dmamux::Dmamux {
                pac::$peri
            }
        }
        impl DmaMux for peripherals::$peri {}
    };
}

peripherals! {
    (bdma, DMA1) => {
        bdma_channels! {
            ($channel_peri:ident, DMA1, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA1, 0);
            };
        }
    };
    (bdma, DMA2) => {
        bdma_channels! {
            ($channel_peri:ident, DMA2, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA2, 1);
            };
        }
    };
    (bdma, BDMA) => {
        bdma_channels! {
            ($channel_peri:ident, BDMA, $channel_num:expr) => {
                impl_dma_channel!($channel_peri, DMAMUX1, $channel_num, DMA2, 1);
            };
        }
    };
    (dmamux, DMAMUX1) => {
        impl_dmamux!(DMAMUX1);
    };
}

#[allow(unused)]
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
