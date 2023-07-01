#![macro_use]

use crate::{pac, peripherals};

pub(crate) fn configure_dmamux<M: MuxChannel>(channel: &mut M, request: u8) {
    let ch_mux_regs = channel.mux_regs().ccr(channel.mux_num());
    ch_mux_regs.write(|reg| {
        reg.set_nbreq(0);
        reg.set_dmareq_id(request);
    });

    ch_mux_regs.modify(|reg| {
        reg.set_ege(true);
    });
}

pub(crate) mod dmamux_sealed {
    use super::*;
    pub trait MuxChannel {
        fn mux_regs(&self) -> pac::dmamux::Dmamux;
        fn mux_num(&self) -> usize;
    }
}

pub struct DMAMUX1;
#[cfg(stm32h7)]
pub struct DMAMUX2;

pub trait MuxChannel: dmamux_sealed::MuxChannel {
    type Mux;
}

foreach_dma_channel! {
    ($channel_peri:ident, $dma_peri:ident, $version:ident, $channel_num:expr, $index:expr, {dmamux: $dmamux:ident, dmamux_channel: $dmamux_channel:expr}) => {
        impl dmamux_sealed::MuxChannel for peripherals::$channel_peri {
            fn mux_regs(&self) -> pac::dmamux::Dmamux {
                pac::$dmamux
            }
            fn mux_num(&self) -> usize {
                $dmamux_channel
            }
        }
        impl MuxChannel for peripherals::$channel_peri {
            type Mux = $dmamux;
        }
    };
}

/// safety: must be called only once
pub(crate) unsafe fn init() {
    crate::_generated::init_dmamux();
}
